//! Representation of the k8s API server

use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::rc::Rc;
use dslab_core::cast;
use crate::pod::Pod;
use dslab_core::context::SimulationContext;
use dslab_core::event::Event;
use dslab_core::handler::EventHandler;
use crate::node::Node;

pub struct APIServer {
    pod_queue: BinaryHeap<Pod>,
    nodes: Rc<RefCell<Vec<Node>>>,
    ctx: SimulationContext,
}

impl APIServer {
    pub fn new() {

    }

    /// Add pod to the PodQueue
    pub fn schedule_pod(&mut self, pod: Pod) {
        self.pod_queue.push(pod);
    }

    /// Pop next pod in the PodQueue
    pub fn get_pod(&mut self) -> Option<Pod> {
        self.pod_queue.pop()
    }

    /// Add node to the nodes (maybe recover old node, maybe horizontal autoscaling)
    pub fn add_node(&mut self) {

    }

    /// Remove node from the nodes (maybe crash old node, maybe horizontal autoscaling)
    pub fn remove_node(&mut self) {

    }

    /// Get list of working nodes
    pub fn get_nodes(&self) -> Rc<RefCell<Vec<Node>>> {
        self.nodes.clone()
    }
}

impl EventHandler for APIServer {
    fn on(&mut self, event: Event) {
        // processing of APIServer events
    }
}