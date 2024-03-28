//! Representation of the k8s node

use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use dslab_core::context::SimulationContext;
use std::collections::HashMap;
use std::rc::Rc;
use std::slice::IterMut;
use dslab_core::{cast, Event, EventHandler};
use serde::Serialize;
use crate::api_server::APIServer;
use crate::events::assigning::{PodAssigningRequest, PodMigrationSucceeded, PodPlacementFailed, PodPlacementRequest, PodPlacementSucceeded};
use crate::events::node::UpdatePodsResources;
use crate::events::pod::PodRequestAndLimitsChange;
use crate::pod::Pod;
use crate::simulation_config::SimulationConfig;

const UPDATE_PODS_RESOURCES_PERIOD: f64 = 10.0;

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
    pub pods: HashMap<u64, Pod>,
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
        ctx.emit(UpdatePodsResources{}, ctx.id(), UPDATE_PODS_RESOURCES_PERIOD);
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

    pub fn add_pod(&mut self, mut pod: Pod) -> Option<Pod> {
        if self.get_free_cpu() < pod.requested_cpu || self.get_free_memory() < pod.requested_memory {
            return Some(pod);
        }
        pod.start_time = self.ctx.time();

        pod.cpu = pod.get_wanted_cpu(self.ctx.time())
            .min(pod.limit_cpu as f64)
            .min(self.get_free_cpu() as f64) as f32;
        pod.memory =  pod.get_wanted_memory(self.ctx.time())
            .min(pod.limit_memory)
            .min(self.get_free_memory());

        self.cpu_load += pod.cpu.max(pod.requested_cpu);
        self.memory_load += pod.memory.max(pod.requested_memory);

        self.pods.insert(pod.id, pod);
        None
    }

    pub fn remove_pod(&mut self, pod_id: u64) -> Pod {
        let pod = self.pods.get(&pod_id).unwrap();
        self.cpu_load -= pod.cpu.max(pod.requested_cpu);
        self.memory_load -= pod.memory.max(pod.requested_memory);

        let mut pod = self.pods.remove(&pod_id).unwrap();
        pod.cpu = 0.0;
        pod.memory = 0.0;
        pod
    }

    pub fn update_pods_resources(&mut self) {
        for (_, pod) in self.pods.iter_mut() {
            let wanted_cpu = pod.get_wanted_cpu(self.ctx.time()).min(pod.limit_cpu as f64);
            let free_cpu = (self.cpu_total as f32) - self.cpu_load;
            let new_cpu = pod.cpu + ((wanted_cpu as f32) - pod.cpu).min(free_cpu);
            self.cpu_load = self.cpu_load - pod.cpu.max(pod.requested_cpu) + new_cpu.max(pod.requested_cpu);
            pod.cpu = new_cpu;

            let wanted_memory = pod.get_wanted_memory(self.ctx.time()).min(pod.limit_memory);
            let free_memory = (self.memory_total as f64) - self.memory_load;
            let new_memory = pod.memory + (wanted_memory - pod.memory).min(free_memory);
            self.memory_load = self.memory_load - pod.memory.max(pod.requested_memory) + new_memory.max(pod.requested_memory);
            pod.memory = new_memory;
        }
    }
}

impl EventHandler for Node {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            PodPlacementRequest { pod, node_id } => {
                let pod_id = pod.id;
                let add_pod_res = self.add_pod(pod);
                if add_pod_res.is_none() {
                    self.ctx.emit(PodPlacementSucceeded { pod_id, node_id }, self.api_server.borrow().id,
                        self.sim_config.message_delay);
                } else {
                    self.ctx.emit(PodPlacementFailed { pod: add_pod_res.unwrap(), node_id },
                        self.api_server.borrow().id, self.sim_config.message_delay);
                }
            }
            UpdatePodsResources {} => {
                self.update_pods_resources();
                self.ctx.emit(UpdatePodsResources{}, self.id, UPDATE_PODS_RESOURCES_PERIOD);
            }
            PodRequestAndLimitsChange { pod_id, new_requested_cpu, new_limit_cpu,
                new_requested_memory, new_limit_memory } => {
                let mut pod = self.remove_pod(pod_id);
                pod.requested_cpu = new_requested_cpu;
                pod.limit_cpu = new_limit_cpu;
                pod.requested_memory = new_requested_memory;
                pod.limit_memory = new_limit_memory;

                self.ctx.emit(PodAssigningRequest {pod}, self.api_server.borrow().id,
                    self.sim_config.message_delay);
            }
        })
    }
}