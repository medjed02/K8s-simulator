//! Representation of the k8s node

use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use dslab_core::context::SimulationContext;
use std::collections::HashMap;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler};
use serde::Serialize;
use crate::api_server::APIServer;
use crate::events::assigning::{PodAssigningRequest, PodMigrationRequest, PodMigrationSucceeded, PodPlacementFailed, PodPlacementRequest, PodPlacementSucceeded};
use crate::events::node::UpdatePodsResources;
use crate::events::pod::PodRequestAndLimitsChange;
use crate::pod::Pod;
use crate::simulation_config::SimulationConfig;

const UPDATE_PODS_RESOURCES_PERIOD: f64 = 300.0;

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
    pub cpu_total: f32,
    pub memory_total: f64,
    pub cpu_allocated: f32,
    pub memory_allocated: f64,
    pub cpu_used: f32,
    pub memory_used: f64,
    pub state: NodeState,
    pub pods: HashMap<u64, Pod>,

    pub memory_overuse_count: u64,

    pub api_server: Rc<RefCell<APIServer>>,
    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl Node {
    pub fn new(
        cpu_total: f32,
        memory_total: f64,
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
            cpu_allocated: 0.0,
            memory_allocated: 0.0,
            cpu_used: 0.0,
            memory_used: 0.0,
            state,
            pods: HashMap::new(),
            memory_overuse_count: 0,
            api_server,
            ctx,
            sim_config
        }
    }

    pub fn get_free_cpu(&self) -> f32 {
        (self.cpu_total as f32) - self.cpu_allocated
    }

    pub fn get_free_memory(&self) -> f64 {
        (self.memory_total as f64) - self.memory_allocated
    }

    pub fn get_cpu_utilization(&self) -> f64 {
        (self.get_free_cpu() as f64) / (self.cpu_total as f64)
    }

    pub fn get_memory_utilization(&self) -> f64 {
        self.get_free_memory() / (self.memory_total as f64)
    }

    pub fn add_pod(&mut self, mut pod: Pod) -> Option<Pod> {
        let wanted_memory=  pod.get_wanted_memory(self.ctx.time()).min(pod.limit_memory);
        if self.get_free_cpu() < pod.requested_cpu || self.get_free_memory() < wanted_memory {
            return Some(pod);
        }

        pod.start_time = self.ctx.time();

        pod.cpu = pod.get_wanted_cpu(self.ctx.time())
            .min(pod.limit_cpu as f64)
            .min(self.get_free_cpu() as f64) as f32;
        pod.memory = wanted_memory;


        self.cpu_used += pod.cpu;
        self.memory_used += pod.memory;
        self.cpu_allocated += pod.cpu.max(pod.requested_cpu);
        self.memory_allocated += pod.memory.max(pod.requested_memory);

        self.pods.insert(pod.id, pod);
        None
    }

    pub fn remove_pod(&mut self, pod_id: u64) -> Option<Pod> {
        let pod = self.pods.get(&pod_id);
        if pod.is_none() {
            return None;
        }
        let pod = pod.unwrap();

        self.cpu_used -= pod.cpu;
        self.memory_used -= pod.memory;
        self.cpu_allocated -= pod.cpu.max(pod.requested_cpu);
        self.memory_allocated -= pod.memory.max(pod.requested_memory);

        let mut pod = self.pods.remove(&pod_id).unwrap();
        pod.cpu = 0.0;
        pod.memory = 0.0;
        Some(pod)
    }

    pub fn can_place_pod(&self, requested_cpu: f32, requested_memory: f64) -> bool {
        self.get_free_cpu() >= requested_cpu && self.get_free_memory() >= requested_memory &&
            !self.is_under_pressure(self.memory_allocated + requested_memory)
    }

    fn update_pods_resources(&mut self) {
        let mut pods_to_evict = Vec::default();
        for (_, pod) in self.pods.iter_mut() {
            let wanted_memory = pod.get_wanted_memory(self.ctx.time()).min(pod.limit_memory);
            if wanted_memory > pod.requested_memory {
                self.memory_overuse_count += 1;
            }
            let free_memory = (self.memory_total as f64) - self.memory_allocated;
            if wanted_memory - pod.memory.max(pod.requested_memory) > free_memory {
                pods_to_evict.push(pod.id);
                continue;
            } else {
                self.memory_allocated = self.memory_allocated - pod.memory.max(pod.requested_memory) + wanted_memory.max(pod.requested_memory);
                self.memory_used = self.memory_used - pod.memory + wanted_memory;
                pod.memory = wanted_memory;
            }

            let wanted_cpu = pod.get_wanted_cpu(self.ctx.time()).min(pod.limit_cpu as f64);
            let free_cpu = (self.cpu_total as f32) - self.cpu_allocated;
            let new_cpu = pod.cpu + ((wanted_cpu as f32) - pod.cpu).min(free_cpu);
            self.cpu_allocated = self.cpu_allocated - pod.cpu.max(pod.requested_cpu) + new_cpu.max(pod.requested_cpu);
            self.cpu_used = self.cpu_used - pod.cpu + new_cpu;
            pod.cpu = new_cpu;
        }

        for pod_id in pods_to_evict {
            self.evict_pod(pod_id);
        }
    }

    fn is_under_pressure(&self, memory_allocated: f64) -> bool {
        memory_allocated >= self.memory_total * self.sim_config.memory_pressure_threshold
    }

    fn evict_pod(&mut self, pod_id: u64) {
        let pod = self.remove_pod(pod_id).unwrap();
        self.ctx.emit(PodMigrationRequest { pod, source_node_id: self.id },
                      self.api_server.borrow().id, self.sim_config.message_delay);
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
                if pod.is_none() {
                    return;
                }
                let mut pod = pod.unwrap();

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