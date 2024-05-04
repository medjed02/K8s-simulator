use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use dslab_core::context::SimulationContext;
use dslab_core::simulation::Simulation;
use crate::api_server::APIServer;
use crate::scheduler::Scheduler;
use crate::simulation_config::SimulationConfig;
use sugars::{rc, refcell};
use crate::cluster_autoscaler::ClusterAutoscaler;
use crate::cluster_autoscaler_algorithm::ClusterAutoscalerAlgorithm;
use crate::dataset_reader::DatasetReader;
use crate::deployment::{Deployment, PodTemplate};
use crate::events::api_server::PodRemoveRequest;
use crate::events::assigning::PodAssigningRequest;
use crate::events::autoscaler::{ClusterAutoscalerScan, HorizontalAutoscalerCycle, MetricsServerSnapshot, VerticalAutoscalerCycle};
use crate::events::deployment::DeploymentCreateRequest;
use crate::events::node::NodeStatusChanged;
use crate::horizontal_autoscaler::HorizontalAutoscaler;
use crate::horizontal_autoscaler_algorithm::HorizontalAutoscalerAlgorithm;
use crate::load_model::LoadModel;
use crate::logger::Logger;
use crate::metrics_server::MetricsServer;
use crate::node::{Node, NodeState};
use crate::pod::{Pod, PodStatus};
use crate::scheduler_algorithm::SchedulerAlgorithm;
use crate::simulation_metrics::MetricsLogger;
use crate::vertical_autoscaler::VerticalAutoscaler;
use crate::vertical_autoscaler_algorithm::VerticalAutoscalerAlgorithm;

pub struct K8sSimulation {
    scheduler: Rc<RefCell<Scheduler>>,
    api_server: Rc<RefCell<APIServer>>,
    cluster_autoscaler: Option<Rc<RefCell<ClusterAutoscaler>>>,
    metrics_server: Option<Rc<RefCell<MetricsServer>>>,
    vertical_autoscaler: Option<Rc<RefCell<VerticalAutoscaler>>>,
    horizontal_autoscaler: Option<Rc<RefCell<HorizontalAutoscaler>>>,
    
    sim: Simulation,
    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,

    last_node_id: u64,
}

impl K8sSimulation {
    /// Creates a simulation with specified config.
    pub fn new(mut sim: Simulation, metrics_logger: Box<dyn MetricsLogger>, logger: Box<dyn Logger>,
               sim_config: SimulationConfig, scheduler_algorithm: Box<dyn SchedulerAlgorithm>,
               cluster_autoscaler_algorithm: Option<Box<dyn ClusterAutoscalerAlgorithm>>,
               vertical_autoscaler_algorithm: Option<Box<dyn VerticalAutoscalerAlgorithm>>,
               horizontal_autoscaler_algorithm: Option<Box<dyn HorizontalAutoscalerAlgorithm>>) -> Self {
        let sim_config = rc!(sim_config);

        let api_server = rc!(refcell!(
            APIServer::new(sim.create_context("api_server"), sim_config.clone(), metrics_logger)
        ));
        sim.add_handler("api_server", api_server.clone());

        let scheduler = rc!(refcell!(Scheduler::new(api_server.clone(), scheduler_algorithm,
                sim.create_context("scheduler"), sim_config.clone())));
        sim.add_handler("scheduler", scheduler.clone());
        {
            api_server.borrow_mut().set_scheduler(scheduler.clone());
        }

        let ctx = sim.create_context("simulation");

        let mut cluster_autoscaler_option = None;
        if cluster_autoscaler_algorithm.is_some() {
            let mut cloud_nodes_pool = Vec::<Rc<RefCell<Node>>>::default();
            for i in 0..sim_config.cloud_nodes_count {
                let name = format!("cloud_node_{}", i);
                let node_ctx = sim.create_context(&name);
                let cpu = sim_config.default_node.cpu;
                let memory = sim_config.default_node.memory;
                let node = rc!(refcell!(Node::new(cpu, memory, NodeState::Working,
                    api_server.clone(), node_ctx, sim_config.clone())));
                cloud_nodes_pool.push(node.clone());
                sim.add_handler(name, node.clone());
            }
            let cluster_ctx = sim.create_context("cluster_autoscaler");
            let cluster_autoscaler = rc!(refcell!(ClusterAutoscaler::new(
                cloud_nodes_pool, api_server.clone(), scheduler.clone(),
                cluster_autoscaler_algorithm.unwrap(), cluster_ctx, sim_config.clone()
            )));
            sim.add_handler("cluster_autoscaler", cluster_autoscaler.clone());
            cluster_autoscaler_option = Some(cluster_autoscaler.clone());

            ctx.emit(ClusterAutoscalerScan{}, cluster_autoscaler.borrow().id, 0.0);
        }

        let mut metrics_server_option = None;
        if vertical_autoscaler_algorithm.is_some() || horizontal_autoscaler_algorithm.is_some() {
            let metrics_server_ctx = sim.create_context("metrics_server");
            let metrics_server = rc!(refcell!(
                MetricsServer::new(api_server.clone(), metrics_server_ctx, sim_config.clone())));
            sim.add_handler("metrics_server", metrics_server.clone());
            metrics_server_option = Some(metrics_server.clone());

            api_server.borrow_mut().set_metrics_server(metrics_server.clone());

            ctx.emit(MetricsServerSnapshot{}, metrics_server.borrow().id, 0.0);
        }

        let mut vertical_autoscaler_option = None;
        if vertical_autoscaler_algorithm.is_some() {
            let vertical_ctx = sim.create_context("vertical_autoscaler");
            let vertical_autoscaler = rc!(refcell!(
                VerticalAutoscaler::new(api_server.clone(), metrics_server_option.clone().unwrap(),
                    vertical_autoscaler_algorithm.unwrap(), vertical_ctx, sim_config.clone())));
            sim.add_handler("vertical_autoscaler", vertical_autoscaler.clone());
            vertical_autoscaler_option = Some(vertical_autoscaler.clone());

            ctx.emit(VerticalAutoscalerCycle {}, vertical_autoscaler.borrow().id, 0.0);
        }

        let mut horizontal_autoscaler_option = None;
        if horizontal_autoscaler_algorithm.is_some() {
            let horizontal_ctx = sim.create_context("horizontal_autoscaler");
            let horizontal_autoscaler = rc!(refcell!(
                HorizontalAutoscaler::new(api_server.clone(), metrics_server_option.clone().unwrap(),
                    horizontal_autoscaler_algorithm.unwrap(), horizontal_ctx, sim_config.clone())
            ));
            sim.add_handler("horizontal_autoscaler", horizontal_autoscaler.clone());
            horizontal_autoscaler_option = Some(horizontal_autoscaler.clone());

            ctx.emit(HorizontalAutoscalerCycle {}, horizontal_autoscaler.borrow().id, 0.0);
        }

        let mut sim = Self {
            scheduler,
            api_server,
            cluster_autoscaler: cluster_autoscaler_option,
            metrics_server: metrics_server_option,
            vertical_autoscaler: vertical_autoscaler_option,
            horizontal_autoscaler: horizontal_autoscaler_option,
            sim,
            ctx,
            sim_config,
            last_node_id: 0
        };

        for node_config in sim.sim_config.nodes.clone() {
            for _ in 0..node_config.count {
                sim.add_node(node_config.cpu, node_config.memory);
            }
        }

        if sim.sim_config.trace.is_some() {
            let mut dataset = DatasetReader::new();
            dataset.parse(  sim.sim_config.trace.as_ref().unwrap().path.clone());

            for node in dataset.node_requests.iter() {
                sim.add_node(node.cpu, node.memory);
            }

            while !dataset.pod_requests.is_empty() {
                let mut pod = dataset.pod_requests.pop().unwrap();
                sim.submit_pod(pod.requested_cpu, pod.requested_memory,
                               pod.limit_cpu, pod.limit_memory, pod.priority_weight,
                               pod.cpu_load_model, pod.memory_load_model, pod.timestamp);
            }
        }

        sim
    }

    /// Add new node to the k8s cluster, return node_id
    pub fn add_node(&mut self, cpu_total: f32, memory_total: f64) -> u32 {
        self.last_node_id += 1;
        let name = format!("node_{}", self.last_node_id);
        let node_ctx = self.sim.create_context(&name);
        let node = rc!(refcell!(Node::new(cpu_total, memory_total, NodeState::Working,
            self.api_server.clone(), node_ctx, self.sim_config.clone())));
        let node_id = node.borrow().id;
        self.sim.add_handler(name, node.clone());
        self.api_server.borrow_mut().add_new_node(node.clone());
        node_id
    }

    pub fn recover_node(&self, node_id: u32, delay: f64) {
        self.ctx.emit(NodeStatusChanged { node_id, new_status: NodeState::Working },
                      self.api_server.borrow().id, self.sim_config.control_plane_message_delay + delay);
    }

    pub fn crash_node(&self, node_id: u32, delay: f64) {
        self.ctx.emit(NodeStatusChanged { node_id, new_status: NodeState::Failed },
                      self.api_server.borrow().id, self.sim_config.control_plane_message_delay + delay);
    }

    pub fn submit_pod(&mut self, requested_cpu: f32, requested_memory: f64, limit_cpu: f32,
                      limit_memory: f64, priority_weight: u64,
                      cpu_load_model: Box<dyn LoadModel>,
                      memory_load_model: Box<dyn LoadModel>,
                      delay: f64) -> u64 {
        let id = self.api_server.borrow_mut().generate_pod_id();
        let pod = Pod::new(id, cpu_load_model, memory_load_model, requested_cpu,
                           requested_memory, limit_cpu, limit_memory, priority_weight,
                           PodStatus::Pending);
        self.ctx.emit(PodAssigningRequest { pod }, self.api_server.borrow().id, delay);
        id
    }

    pub fn submit_deployment(&mut self, requested_cpu: f32, requested_memory: f64, limit_cpu: f32,
                             limit_memory: f64, priority_weight: u64,
                             cpu_load_model: Box<dyn LoadModel>,
                             memory_load_model: Box<dyn LoadModel>,
                             cnt_replicas: u64,
                             delay: f64) -> u64 {
        let id = self.api_server.borrow_mut().generate_deployment_id();
        let pod_template = PodTemplate {
            cpu_load_model, memory_load_model,
            requested_cpu, requested_memory,
            limit_cpu, limit_memory,
            priority_weight
        };
        let deployment = Deployment::new(id, pod_template, cnt_replicas);
        self.ctx.emit(DeploymentCreateRequest { deployment }, self.api_server.borrow().id, delay);
        id
    }

    pub fn remove_pod(&self, pod_id: u64) {
        self.ctx.emit(PodRemoveRequest { pod_id }, self.api_server.borrow().id,
                      self.sim_config.message_delay);
    }

    /// Returns the map with references to working nodes.
    pub fn working_nodes(&self) -> BTreeMap<u32, Rc<RefCell<Node>>> {
        self.api_server.borrow().working_nodes.clone()
    }

    /// Returns the map with references to failed nodes.
    pub fn failed_nodes(&self) -> BTreeMap<u32, Rc<RefCell<Node>>> {
        self.api_server.borrow().failed_nodes.clone()
    }

    /// Returns the reference to node (node status, resources).
    pub fn node(&self, node_id: u32) -> Rc<RefCell<Node>> {
        let api_server_ref = self.api_server.borrow();
        let node = api_server_ref.working_nodes.get(&node_id);
        if node.is_some() {
            node.unwrap().clone()
        } else {
            api_server_ref.failed_nodes.get(&node_id).unwrap().clone()
        }
    }

    /// Returns the average allocated CPU across all working nodes.
    pub fn average_cpu_allocated(&self) -> f64 {
        self.api_server.borrow().average_cpu_allocated()
    }

    /// Returns the average allocated memory across all working nodes.
    pub fn average_memory_allocated(&self) -> f64 {
        self.api_server.borrow().average_cpu_allocated()
    }

    /// Returns the current allocated CPU load rate (% of overall CPU used).
    pub fn cpu_allocated_load_rate(&self) -> f64 {
        self.api_server.borrow().cpu_allocated_load_rate()
    }

    /// Returns the current allocated memory load rate (% of overall RAM used).
    pub fn memory_allocated_load_rate(&self) -> f64 {
        self.api_server.borrow().memory_allocated_load_rate()
    }

    /// Returns the average used CPU across all working nodes.
    pub fn average_cpu_used(&self) -> f64 {
        self.api_server.borrow().average_cpu_used()
    }

    /// Returns the average used memory across all working nodes.
    pub fn average_memory_used(&self) -> f64 {
        self.api_server.borrow().average_memory_used()
    }

    /// Returns the current used CPU load rate (% of overall CPU used).
    pub fn cpu_used_load_rate(&self) -> f64 {
        self.api_server.borrow().cpu_used_load_rate()
    }

    /// Returns the current used memory load rate (% of overall RAM used).
    pub fn memory_used_load_rate(&self) -> f64 {
        self.api_server.borrow().memory_used_load_rate()
    }

    pub fn finish_simulation(&self, path: &str) -> Result<(), std::io::Error> {
        self.api_server.borrow_mut().finish_and_save_log_metrics(path)
    }

    /// Performs the specified number of steps through the simulation (see dslab-core docs).
    pub fn steps(&mut self, step_count: u64) -> bool {
        self.sim.steps(step_count)
    }

    /// Steps through the simulation until there are no pending events left.
    pub fn step_until_no_events(&mut self) {
        self.sim.step_until_no_events();
    }

    /// Steps through the simulation with duration limit (see dslab-core docs).
    pub fn step_for_duration(&mut self, time: f64) {
        self.sim.step_for_duration(time);
    }

    /// Steps through the simulation until the specified time (see dslab-core docs).
    pub fn step_until_time(&mut self, time: f64) {
        self.sim.step_until_time(time);
    }

    /// Returns the total number of created events.
    pub fn event_count(&self) -> u64 {
        self.sim.event_count()
    }

    /// Returns the current simulation time.
    pub fn current_time(&self) -> f64 {
        self.sim.time()
    }
}