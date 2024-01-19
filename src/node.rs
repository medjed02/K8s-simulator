//! Representation of the k8s node

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use dslab_core::context::SimulationContext;
use std::collections::HashMap;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler};
use serde::Serialize;
use crate::api_server::APIServer;
use crate::events::assigning::{PodMigrationSucceeded, PodPlacementFailed, PodPlacementRequest, PodPlacementSucceeded};
use crate::pod::Pod;
use crate::simulation_config::SimulationConfig;

/// Node state (for imitation crash of the node)
#[derive(Clone, Debug, PartialEq, Serialize)]
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

pub struct Node {
    pub id: u32,
    pub cpu_total: u32,
    pub memory_total: u64,
    pub cpu_load: f32,
    pub memory_load: f64,
    pub state: NodeState,
    pods: HashMap<u64, Pod>,
    pub api_server: Rc<RefCell<APIServer>>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl Node {
    pub fn new(
        cpu_total: u32,
        memory_total: u64,
        cpu_load: f32,
        memory_load: f64,
        state: NodeState,
        api_server: Rc<RefCell<APIServer>>,
        ctx: SimulationContext,
        sim_config: Rc<SimulationConfig>,
    ) -> Self {
        Self {
            id: ctx.id(),
            cpu_total,
            memory_total,
            cpu_load,
            memory_load,
            state,
            pods: HashMap::new(),
            api_server,
            ctx,
            sim_config
        }
    }

    pub fn get_free_cpu(&self) -> f32 {
        (self.cpu_total as f32) - self.cpu_load
    }

    pub fn get_free_memory(&self) -> f64 {
        (self.memory_total as f64) - self.memory_load
    }

    pub fn get_cpu_utilization(&self) -> f64 {
        (self.get_free_cpu() as f64) / (self.cpu_total as f64)
    }

    pub fn get_memory_utilization(&self) -> f64 {
        self.get_free_memory() / (self.memory_total as f64)
    }

    pub fn add_pod(&mut self, pod: Pod) -> bool {
        if self.get_free_cpu() < pod.requested_cpu || self.get_free_memory() < pod.requested_memory {
            return false;
        }
        self.cpu_load += pod.requested_cpu;
        self.memory_load += pod.requested_memory;
        self.pods.insert(pod.id, pod);
        true
    }

    pub fn remove_pod(&mut self, pod_id: u64) {
        self.pods.remove(&pod_id);
    }
}

impl EventHandler for Node {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            PodPlacementRequest { pod, node_id } => {
                let pod_id = pod.id;
                if self.add_pod(pod) {
                    self.ctx.emit(PodPlacementSucceeded { pod_id, node_id }, self.api_server.borrow().id,
                        self.sim_config.message_delay);
                } else {
                    self.ctx.emit(PodPlacementFailed { pod_id, node_id }, self.api_server.borrow().id,
                        self.sim_config.message_delay);
                }
            }
        })
    }
}