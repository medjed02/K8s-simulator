//! Representation of the k8s API server

use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap};
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
use crate::events::node::{NewNodeAdded, NodeStatusChanged};
use crate::events::api_server::{GetNodesResponse, GetNodesRequest, GetPodRequest, GetPodResponse};
use crate::events::assigning::PodAssigningRequest;

pub struct APIServer {
    pub pod_queue: BinaryHeap<Pod>,
    pub working_nodes: Rc<RefCell<HashMap<u64, Rc<RefCell<Node>>>>>,
    pub failed_nodes: Rc<RefCell<HashMap<u64, Rc<RefCell<Node>>>>>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl APIServer {
    pub fn new(ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            pod_queue: BinaryHeap::default(),
            working_nodes: rc!(refcell!(HashMap::default())),
            failed_nodes: rc!(refcell!(HashMap::default())),
            ctx,
            sim_config,
        }
    }

    /// Add pod to the PodQueue
    pub fn add_pod(&mut self, pod: Pod) {
        self.pod_queue.push(pod);
    }

    /// Pop next pod in the PodQueue
    pub fn get_pod(&mut self) -> Option<Pod> {
        self.pod_queue.pop()
    }

    /// Add new node to the working nodes
    pub fn add_new_node(&mut self, node: Rc<RefCell<Node>>) {
        node.borrow().state = NodeState::Working;
        self.working_nodes.borrow().insert(node.borrow().id, node.clone());
    }

    /// Recover node from the failed nodes
    pub fn recover_node(&mut self, node_id: u64) {
        let mut node = self.failed_nodes.borrow().remove(&node_id).unwrap();
        node.borrow().state = NodeState::Working;
        self.working_nodes.borrow().insert(node_id, node);
    }

    /// Remove node from the working nodes (maybe crash old node, maybe horizontal autoscaling)
    pub fn remove_node(&mut self, node_id: u64) {
        let node = self.working_nodes.borrow().remove(&node_id).unwrap();
        node.borrow().state = NodeState::Failed;
        self.failed_nodes.borrow().insert(node_id, node);
    }

    /// Get list of working nodes
    pub fn get_working_nodes(&self) -> Rc<RefCell<HashMap<u64, Rc<RefCell<Node>>>>> {
        self.working_nodes.clone()
    }

    /// Returns the average CPU load across all working nodes.
    pub fn average_cpu_load(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.borrow().into_iter() {
            sum_cpu_load += node.borrow().cpu_load;
        }
        sum_cpu_load / (self.working_nodes.borrow().len() as f64)
    }

    /// Returns the average memory load across all working nodes.
    pub fn average_memory_load(&self) -> f64 {
        let mut sum_cpu_load: f64 = 0.0;
        for (node_id, node) in self.working_nodes.borrow().into_iter() {
            sum_cpu_load += node.borrow().memory_load;
        }
        sum_cpu_load / (self.working_nodes.borrow().len() as f64)
    }

    /// Returns the current CPU load rate (% of overall CPU used).
    pub fn cpu_load_rate(&self) -> f64 {
        let mut sum_cpu_load_rate: f64 = 0.0;
        for (node_id, node) in self.working_nodes.borrow().into_iter() {
            sum_cpu_load_rate += node.borrow().cpu_load / (node.borrow().cpu_total as f64);
        }
        sum_cpu_load_rate / (self.working_nodes.borrow().len() as f64)
    }

    /// Returns the current memory load rate (% of overall RAM used).
    pub fn memory_load_rate(&self) -> f64 {
        let mut sum_cpu_memory_rate: f64 = 0.0;
        for (node_id, node) in self.working_nodes.borrow().into_iter() {
            sum_cpu_memory_rate += node.borrow().memory_load / (node.borrow().memory_total as f64);
        }
        sum_cpu_memory_rate / (self.working_nodes.borrow().len() as f64)
    }
}

impl EventHandler for APIServer {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            NewNodeAdded { node } => {
                self.add_new_node(node);
            }
            NodeStatusChanged { node_id, new_status } => {
                if new_status == NodeState::Working {
                    self.recover_node(node_id);
                } else {
                    self.remove_node(node_id);
                }
            }
            GetNodesRequest {} => {
                self.ctx.emit(GetNodesResponse { nodes: self.get_working_nodes() }, event.src,
                    self.sim_config.control_plane_message_delay);
            }
            GetPodRequest {} => {
                self.ctx.emit(GetPodResponse { pod: self.get_pod() }, event.src,
                    self.sim_config.control_plane_message_delay);
            }
            PodAssigningRequest { pod } => {
                self.add_pod(pod);
            }
        })
    }
}