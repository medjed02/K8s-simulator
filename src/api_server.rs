//! Representation of the k8s API server

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::rc::Rc;
use dslab_core::cast;
use crate::pod::Pod;
use dslab_core::context::SimulationContext;
use dslab_core::event::Event;
use dslab_core::handler::EventHandler;
use rand::distributions::uniform::SampleBorrow;
use crate::node::{Node, NodeState};
use crate::simulation_config::SimulationConfig;
use sugars::{rc, refcell};
use crate::deployment::Deployment;
use crate::events::node::{AllocateNewDefaultNodes, NodeStatusChanged, RemoveNode};
use crate::events::assigning::{PodAssigningRequest, PodAssigningSucceeded, PodAssigningFailed, PodPlacementRequest, PodPlacementSucceeded, PodPlacementFailed, PodMigrationRequest};
use crate::events::api_server::PodRemoveRequest;
use crate::events::deployment::{DeploymentCreateRequest, DeploymentHorizontalAutoscaling};
use crate::events::logger::MetricsSnapshot;
use crate::events::scheduler::MoveRequest;
use crate::metrics_server::MetricsServer;
use crate::scheduler::Scheduler;
use crate::simulation_metrics::{Metrics, MetricsLogger};

pub struct APIServer {
    pub id: u32,
    pub working_nodes: BTreeMap<u32, Rc<RefCell<Node>>>,
    pub failed_nodes: BTreeMap<u32, Rc<RefCell<Node>>>,
    pub pod_to_node_map: HashMap<u64, u32>,
    pub deployment_to_replicas: HashMap<u64, Vec<u64>>,
    pub deployments_start_time: HashMap<u64, f64>,
    pub deployments: HashMap<u64, Deployment>,

    scheduler: Option<Rc<RefCell<Scheduler>>>,
    metrics_server: Option<Rc<RefCell<MetricsServer>>>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,

    metrics_logger: Box<dyn MetricsLogger>,
    pod_migration_count: u64,

    pod_counter: u64,
    deployment_counter: u64,
}

impl APIServer {
    pub fn new(ctx: SimulationContext, sim_config: Rc<SimulationConfig>,
               metrics_logger: Box<dyn MetricsLogger>) -> Self {
        if metrics_logger.snapshot_period() > 0.0 {
            ctx.emit(MetricsSnapshot {}, ctx.id(), metrics_logger.snapshot_period());
        }

        Self {
            id: ctx.id(),
            working_nodes: BTreeMap::default(),
            failed_nodes: BTreeMap::default(),
            pod_to_node_map: HashMap::default(),
            deployment_to_replicas: HashMap::default(),
            deployments_start_time: HashMap::default(),
            deployments: HashMap::default(),
            scheduler: None,
            metrics_server: None,
            ctx,
            sim_config,
            metrics_logger,
            pod_migration_count: 0,
            pod_counter: 0,
            deployment_counter: 0,
        }
    }

    pub fn set_scheduler(&mut self, scheduler: Rc<RefCell<Scheduler>>) {
        self.scheduler = Some(scheduler);
    }

    pub fn set_metrics_server(&mut self, metrics_server: Rc<RefCell<MetricsServer>>) {
        self.metrics_server = Some(metrics_server);
    }


    /// Add new node to the working nodes
    pub fn add_new_node(&mut self, node: Rc<RefCell<Node>>) {
        node.borrow_mut().state = NodeState::Working;
        self.working_nodes.insert(node.borrow().id, node.clone());
        self.ctx.emit(MoveRequest {}, self.scheduler.clone().unwrap().borrow().id,
                      self.sim_config.control_plane_message_delay);
    }

    /// Recover node from the failed nodes
    pub fn recover_node(&mut self, node_id: u32) {
        let mut node = self.failed_nodes.remove(&node_id).unwrap();
        node.borrow_mut().state = NodeState::Working;
        self.working_nodes.insert(node_id, node);
    }

    /// Remove node from cluster (from working nodes)
    pub fn remove_node(&mut self, node_id: u32) {
        let node = self.working_nodes.remove(&node_id).unwrap();
        let node = node.borrow_mut();
        for (_, pod) in node.pods.clone().into_iter() {
            self.ctx.emit(PodAssigningRequest { pod }, self.id, 0.0);
        }
    }

    /// Crash node (from working nodes)
    pub fn crash_node(&mut self, node_id: u32) {
        let node = self.working_nodes.remove(&node_id).unwrap();
        let mut mut_node = node.borrow_mut();
        for (pod_id, pod) in mut_node.pods.clone().into_iter() {
            self.ctx.emit(PodAssigningRequest { pod }, self.id, 0.0);
        }
        mut_node.pods.clear();
        mut_node.state = NodeState::Failed;
        mut_node.cpu_allocated = 0.0;
        mut_node.memory_allocated = 0.0;
        mut_node.cpu_used = 0.0;
        mut_node.memory_used = 0.0;
        drop(mut_node);
        self.failed_nodes.insert(node_id, node);
    }

    pub fn remove_pod(&mut self, pod_id: u64) {
        if self.metrics_server.is_some() {
            self.metrics_server.clone().unwrap().borrow_mut().clear_pod_statistics(pod_id);
        }
        let node_id = self.pod_to_node_map.get(&pod_id);
        if node_id.is_some() {
            let node = self.working_nodes.get(node_id.unwrap());
            if node.is_some() {
                node.unwrap().borrow_mut().remove_pod(pod_id);
            }
        }
        self.pod_to_node_map.remove(&pod_id);
    }

    /// Get list of working nodes
    pub fn get_working_nodes(&self) -> &BTreeMap<u32, Rc<RefCell<Node>>> {
        &self.working_nodes
    }

    pub fn get_real_cnt_replicas(&self, deployment_id: u64) -> u64 {
        let replica_ids = self.deployment_to_replicas.get(&deployment_id);
        if replica_ids.is_none() {
            return 1;
        }
        let replica_ids = replica_ids.unwrap();

        let mut replica_cnt: u64 = 0;
        for pod_id in replica_ids {
            if self.pod_to_node_map.get(pod_id).is_none() {
                continue;
            }
            replica_cnt += 1;
        }
        replica_cnt
    }

    pub fn get_deployment_start_time(&self, deployment_id: u64) -> f64 {
        let start_time = self.deployments_start_time.get(&deployment_id);
        if start_time.is_none() {
            return -1.0;
        }
        *start_time.unwrap()
    }

    /// Returns the average allocated CPU across all working nodes.
    pub fn average_cpu_allocated(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_cpu_load += (node.borrow().cpu_allocated as f64);
        }
        sum_cpu_load / (self.working_nodes.len() as f64)
    }

    /// Returns the average allocated memory across all working nodes.
    pub fn average_memory_allocated(&self) -> f64 {
        let mut sum_memory_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_memory_load += node.borrow().memory_allocated;
        }
        sum_memory_load / (self.working_nodes.len() as f64)
    }

    /// Returns the current allocated CPU load rate (% of overall CPU used).
    pub fn cpu_allocated_load_rate(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        let mut sum_cpu_total: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_cpu_load += node.borrow().cpu_allocated as f64;
            sum_cpu_total += node.borrow().cpu_total as f64;
        }
        sum_cpu_load / sum_cpu_total
    }

    /// Returns the current allocated memory load rate (% of overall RAM used).
    pub fn memory_allocated_load_rate(&self) -> f64 {
        let mut sum_memory_load: f64 = 0.0;
        let mut sum_memory_total: f64 = 0.0;
        for (_, node) in self.working_nodes.iter() {
            sum_memory_load += node.borrow().memory_allocated;
            sum_memory_total += node.borrow().memory_total;
        }
        sum_memory_load / sum_memory_total
    }

    /// Returns the average used CPU across all working nodes.
    pub fn average_cpu_used(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_cpu_load += (node.borrow().cpu_used as f64);
        }
        sum_cpu_load / (self.working_nodes.len() as f64)
    }

    /// Returns the average used memory across all working nodes.
    pub fn average_memory_used(&self) -> f64 {
        let mut sum_memory_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_memory_load += node.borrow().memory_used;
        }
        sum_memory_load / (self.working_nodes.len() as f64)
    }

    /// Returns the current used CPU load rate (% of overall CPU used).
    pub fn cpu_used_load_rate(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        let mut sum_cpu_total: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_cpu_load += node.borrow().cpu_used as f64;
            sum_cpu_total += node.borrow().cpu_total as f64;
        }
        sum_cpu_load / sum_cpu_total
    }

    /// Returns the current used memory load rate (% of overall RAM used).
    pub fn memory_used_load_rate(&self) -> f64 {
        let mut sum_memory_load: f64 = 0.0;
        let mut sum_memory_total: f64 = 0.0;
        for (_, node) in self.working_nodes.iter() {
            sum_memory_load += node.borrow().memory_used;
            sum_memory_total += node.borrow().memory_total;
        }
        sum_memory_load / sum_memory_total
    }

    pub fn memory_overuse_count(&self) -> u64 {
        let mut memory_overuse_count: u64 = 0;
        for (_, node) in self.working_nodes.iter() {
            memory_overuse_count += node.borrow().memory_overuse_count;
        }
        memory_overuse_count
    }

    pub fn deployments_cpu_utilization(&self) -> f64 {
        let mut sum_cpu_utilization: f64 = 0.0;
        for (_, deployment) in self.deployments.iter() {
            let replica_ids = self.deployment_to_replicas.get(&deployment.id);
            if replica_ids.is_none() {
                continue
            }
            let replica_ids = replica_ids.unwrap();

            let mut sum_cpu: f64 = 0.0;
            for pod_id in replica_ids {
                let node_id = self.pod_to_node_map.get(pod_id);
                if node_id.is_none() {
                    continue;
                }
                let node = self.working_nodes.get(node_id.unwrap());
                if node.is_none() {
                    continue;
                }
                let node = node.unwrap().borrow();
                let pod = node.pods.get(pod_id);
                if pod.is_some() {
                    sum_cpu += pod.unwrap().cpu as f64;
                }
            }
            sum_cpu /= replica_ids.len() as f64;

            sum_cpu_utilization += sum_cpu / deployment.pod_template.requested_cpu as f64;
        }

        if self.deployments.is_empty() {
            0.0
        } else {
            sum_cpu_utilization / (self.deployments.len() as f64)
        }
    }

    pub fn deployments_memory_utilization(&self) -> f64 {
        let mut sum_memory_utilization: f64 = 0.0;
        for (_, deployment) in self.deployments.iter() {
            let replica_ids = self.deployment_to_replicas.get(&deployment.id);
            if replica_ids.is_none() {
                continue
            }
            let replica_ids = replica_ids.unwrap();

            let mut sum_memory: f64 = 0.0;
            for pod_id in replica_ids {
                let node_id = self.pod_to_node_map.get(pod_id);
                if node_id.is_none() {
                    continue;
                }
                let node = self.working_nodes.get(node_id.unwrap());
                if node.is_none() {
                    continue;
                }
                let node = node.unwrap().borrow();
                let pod = node.pods.get(pod_id);
                if pod.is_some() {
                    sum_memory += pod.unwrap().memory;
                }
            }
            sum_memory /= replica_ids.len() as f64;

            sum_memory_utilization += sum_memory / deployment.pod_template.requested_memory;
        }

        if self.deployments.is_empty() {
            0.0
        } else {
            sum_memory_utilization / (self.deployments.len() as f64)
        }
    }

    pub fn log_metrics(&mut self) {
        let metrics = Metrics::new(
            self.ctx.time(),
            self.average_cpu_allocated(),
            self.average_memory_allocated(),
            self.cpu_allocated_load_rate(),
            self.memory_allocated_load_rate(),
            self.average_cpu_used(),
            self.average_memory_used(),
            self.cpu_used_load_rate(),
            self.memory_used_load_rate(),
            self.pod_migration_count,
            self.memory_overuse_count(),
            self.working_nodes.len() as u64,
            self.deployments_cpu_utilization(),
            self.deployments_memory_utilization(),
            self.pod_to_node_map.len() as u64,
        );
        self.metrics_logger.log_metrics(metrics);
    }

    pub fn finish_and_save_log_metrics(&mut self, path: &str) -> Result<(), std::io::Error>  {
        self.log_metrics();
        self.metrics_logger.save_log(path)
    }

    pub fn generate_pod_id(&mut self) -> u64 {
        self.pod_counter += 1;
        self.pod_counter
    }

    pub fn generate_deployment_id(&mut self) -> u64 {
        self.deployment_counter += 1;
        self.deployment_counter
    }
}

impl EventHandler for APIServer {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            NodeStatusChanged { node_id, new_status } => {
                if new_status == NodeState::Working {
                    self.recover_node(node_id);
                } else {
                    self.crash_node(node_id);
                }
                self.ctx.emit(MoveRequest {}, self.scheduler.clone().unwrap().borrow().id,
                              self.sim_config.control_plane_message_delay);
            }
            PodAssigningRequest { pod } => {
                self.scheduler.clone().unwrap().borrow_mut().add_pod(pod);
            }
            PodAssigningSucceeded { pod, node_id } => {
                if self.working_nodes.get(&node_id).is_none() {
                    self.scheduler.clone().unwrap().borrow_mut().add_pod(pod);
                } else {
                    self.ctx.emit(PodPlacementRequest { pod, node_id },
                        self.working_nodes.get(&node_id).unwrap().borrow().id,
                        self.sim_config.message_delay);
                }
            }
            PodPlacementSucceeded { pod_id, node_id } => {
                self.pod_to_node_map.insert(pod_id, node_id);
            }
            PodPlacementFailed { pod, node_id } => {
                self.scheduler.clone().unwrap().borrow_mut().add_pod(pod);
            }
            PodRemoveRequest { pod_id } => {
                self.remove_pod(pod_id);
            }
            RemoveNode { node_id } => {
                self.remove_node(node_id);
            }
            DeploymentCreateRequest { deployment } => {
                let mut scheduler = self.scheduler.clone().unwrap();
                let mut replicas = Vec::default();
                for _ in 0..deployment.cnt_replicas {
                    let id = self.generate_pod_id();
                    scheduler.borrow_mut().add_pod(deployment.create_new_replica(id));
                    replicas.push(id);
                }
                self.deployments.insert(deployment.id, deployment.clone());
                self.deployment_to_replicas.insert(deployment.id, replicas);
            }
            DeploymentHorizontalAutoscaling { id, new_cnt_replicas } => {
                let deployment = self.deployments.remove(&id);
                if deployment.is_none() {
                    return;
                }
                let mut deployment = deployment.unwrap();
                let mut replicas = self.deployment_to_replicas.remove(&id).unwrap();
                if new_cnt_replicas < deployment.cnt_replicas {
                    for _ in 0..(deployment.cnt_replicas - new_cnt_replicas) {
                        let pod_id = replicas.pop().unwrap();
                        self.ctx.emit(PodRemoveRequest { pod_id }, self.id, self.sim_config.message_delay);
                    }
                } else {
                    for _ in 0..(new_cnt_replicas - deployment.cnt_replicas) {
                        let id = self.generate_pod_id();
                        self.ctx.emit(PodAssigningRequest { pod: deployment.create_new_replica(id) },
                            self.id, self.sim_config.message_delay);
                        replicas.push(id);
                    }
                }
                deployment.cnt_replicas = new_cnt_replicas;
                self.deployments.insert(deployment.id, deployment.clone());
                self.deployment_to_replicas.insert(id, replicas);
            }
            MetricsSnapshot {} => {
                self.log_metrics();

                if self.metrics_logger.snapshot_period() > 0.0 {
                    self.ctx.emit(MetricsSnapshot {}, self.id, self.metrics_logger.snapshot_period());
                }
            }
            PodMigrationRequest { pod, source_node_id } => {
                self.pod_migration_count += 1;
                self.remove_pod(pod.id);
                self.scheduler.clone().unwrap().borrow_mut().add_pod(pod);
            }
        })
    }
}