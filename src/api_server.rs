//! Representation of the k8s API server

use std::cell::RefCell;
use std::collections::{BinaryHeap, BTreeMap, HashMap};
use std::hash::Hash;
use std::rc::Rc;
use dslab_core::cast;
use crate::pod::Pod;
use dslab_core::context::SimulationContext;
use dslab_core::event::Event;
use dslab_core::handler::EventHandler;
use crate::node::{Node, NodeState};
use crate::simulation_config::SimulationConfig;
use sugars::{rc, refcell};
use crate::events::node::NodeStatusChanged;
use crate::events::assigning::{PodAssigningRequest, PodAssigningSucceeded, PodAssigningFailed, PodPlacementRequest,
                               PodPlacementSucceeded, PodPlacementFailed};
use crate::events::api_server::PodRemoveRequest;
use crate::scheduler::Scheduler;

pub struct APIServer {
    pub id: u32,
    pub working_nodes: BTreeMap<u32, Rc<RefCell<Node>>>,
    pub failed_nodes: BTreeMap<u32, Rc<RefCell<Node>>>,
    pub pod_to_node_map: HashMap<u64, u32>,
    scheduler: Option<Rc<RefCell<Scheduler>>>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl APIServer {
    pub fn new(ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            working_nodes: BTreeMap::default(),
            failed_nodes: BTreeMap::default(),
            pod_to_node_map: HashMap::default(),
            scheduler: None,
            ctx,
            sim_config,
        }
    }

    pub fn set_scheduler(&mut self, scheduler: Rc<RefCell<Scheduler>>) {
        self.scheduler = Some(scheduler);
    }


    /// Add new node to the working nodes
    pub fn add_new_node(&mut self, node: Rc<RefCell<Node>>) {
        node.borrow_mut().state = NodeState::Working;
        self.working_nodes.insert(node.borrow().id, node.clone());
    }

    /// Recover node from the failed nodes
    pub fn recover_node(&mut self, node_id: u32) {
        let mut node = self.failed_nodes.remove(&node_id).unwrap();
        node.borrow_mut().state = NodeState::Working;
        self.working_nodes.insert(node_id, node);
    }

    /// Remove node from the working nodes (maybe crash old node, maybe horizontal autoscaling)
    pub fn remove_node(&mut self, node_id: u32) {
        let node = self.working_nodes.remove(&node_id).unwrap();
        let mut mut_node = node.borrow_mut();
        for (pod_id, pod) in mut_node.pods.clone().into_iter() {
            self.ctx.emit(PodAssigningRequest { pod }, self.id, 0.0);
        }
        mut_node.pods.clear();
        mut_node.state = NodeState::Failed;
        mut_node.cpu_load = 0.0;
        mut_node.memory_load = 0.0;
        drop(mut_node);
        self.failed_nodes.insert(node_id, node);
    }

    /// Get list of working nodes
    pub fn get_working_nodes(&self) -> &BTreeMap<u32, Rc<RefCell<Node>>> {
        &self.working_nodes
    }

    /// Returns the average CPU load across all working nodes.
    pub fn average_cpu_load(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_cpu_load += (node.borrow().cpu_load as f64);
        }
        sum_cpu_load / (self.working_nodes.len() as f64)
    }

    /// Returns the average memory load across all working nodes.
    pub fn average_memory_load(&self) -> f64 {
        let mut sum_memory_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_memory_load += node.borrow().memory_load;
        }
        sum_memory_load / (self.working_nodes.len() as f64)
    }

    /// Returns the current CPU load rate (% of overall CPU used).
    pub fn cpu_load_rate(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        let mut sum_cpu_total: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_cpu_load += (node.borrow().cpu_load as f64);
            sum_cpu_total += (node.borrow().cpu_total as f64)
        }
        sum_cpu_load / sum_cpu_total
    }

    /// Returns the current memory load rate (% of overall RAM used).
    pub fn memory_load_rate(&self) -> f64 {
        let mut sum_memory_load: f64 = 0.0;
        let mut sum_memory_total: f64 = 0.0;
        for (node_id, node) in self.working_nodes.iter() {
            sum_memory_load += (node.borrow().memory_load as f64);
            sum_memory_total += (node.borrow().memory_total as f64)
        }
        sum_memory_load / sum_memory_total
    }
}

impl EventHandler for APIServer {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            NodeStatusChanged { node_id, new_status } => {
                if new_status == NodeState::Working {
                    self.recover_node(node_id);
                } else {
                    self.remove_node(node_id);
                }
            }
            PodAssigningRequest { pod } => {
                self.scheduler.clone().unwrap().borrow_mut().add_pod(pod);
            }
            PodAssigningSucceeded { pod, node_id } => {
                let node_name = format!("node_{}", node_id);
                self.ctx.emit(PodPlacementRequest { pod, node_id },
                    self.working_nodes.get(&node_id).unwrap().borrow().id, self.sim_config.message_delay);
            }
            PodAssigningFailed { pod } => {
            }
            PodPlacementSucceeded { pod_id, node_id } => {
                self.pod_to_node_map.insert(pod_id, node_id);
            }
            PodPlacementFailed { pod_id, node_id } => {
            }
            PodRemoveRequest { pod_id } => {
                let node_id = self.pod_to_node_map.get(&pod_id);
                if node_id.is_some() {
                    self.working_nodes.get(node_id.unwrap()).unwrap().borrow_mut().remove_pod(pod_id);
                }
            }
        })
    }
}