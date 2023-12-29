//! Representation of the k8s node

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use dslab_core::context::SimulationContext;
use std::collections::HashMap;
use crate::pod::Pod;

/// Node state (for imitation crash of the node)
#[derive(Clone, Debug, PartialEq)]
pub enum NodeState {
    Working,
    Failed,
}

impl Display for NodeState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            NodeState::Working => write!(f, "working"),
            NodeState::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub id: u64,
    pub cpu_total: f64,
    pub memory_total: f64,
    pub cpu_load: f64,
    pub memory_load: f64,
    pub state: NodeState,
    pods: HashMap<u64, Pod>,

    ctx: SimulationContext,
}

impl Node {
    pub fn new(
        id: u64,
        cpu_total: f64,
        memory_total: f64,
        cpu_load: f64,
        memory_load: f64,
        state: NodeState,
        ctx: SimulationContext
    ) -> Self {
        Self {
            id,
            cpu_total,
            memory_total,
            cpu_load,
            memory_load,
            state,
            pods: HashMap::new(),
            ctx
        }
    }

    pub fn get_free_cpu(&self) -> f64 {
        self.cpu_total - self.cpu_load
    }

    pub fn get_free_memory(&self) -> f64 {
        self.memory_total - self.memory_load
    }

    pub fn get_cpu_utilization(&self) -> f64 {
        self.get_free_cpu() / self.cpu_total
    }

    pub fn get_memory_utilization(&self) -> f64 {
        self.get_free_memory() / self.memory_total
    }

    pub fn add_pod(&mut self, pod: Pod) -> bool {
        if self.get_free_cpu() < pod.requested_cpu || self.get_free_memory() < pod.requested_memory {
            return false;
        }
        self.pods.insert(pod.id, pod);
        // ctx.submit_pod
        true
    }

    pub fn remove_pod(&mut self, pod_id: u64){
        self.pods.remove(&pod_id);
    }
}