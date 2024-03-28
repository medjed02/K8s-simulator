use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use crate::api_server::APIServer;
use crate::events::autoscaler::MetricsServerSnapshot;
use crate::simulation_config::SimulationConfig;

#[derive(Copy, Clone)]
pub struct PodSnapshot {
    pub cpu: f32,
    pub memory: f64,
    pub snapshot_time: f64,
}

const DEFAULT_LOWER_BOUND_CPU_PERCENTILE: f64 = 0.5;
const DEFAULT_TARGET_CPU_PERCENTILE: f64 = 0.9;
const DEFAULT_UPPER_BOUND_CPU_PERCENTILE: f64 = 0.95;
const DEFAULT_LOWER_BOUND_MEMORY_PERCENTILE: f64 = 0.5;
const DEFAULT_TARGET_MEMORY_PERCENTILE: f64 = 0.9;
const DEFAULT_UPPER_BOUND_MEMORY_PERCENTILE: f64 = 0.95;

#[derive(Clone)]
pub struct PodStatistic {
    pub default_lower_bound_cpu_percentile: f32,
    pub default_target_cpu_percentile: f32,
    pub default_upper_bound_cpu_percentile: f32,

    pub default_lower_bound_memory_percentile: f64,
    pub default_target_memory_percentile: f64,
    pub default_upper_memory_percentile: f64,

    pub resource_history: Vec<PodSnapshot>,
    pub period_time: f64,
}

pub struct MetricsServer {
    pub id: u32,
    pods_resource_history: HashMap<u64, Vec<PodSnapshot>>,
    api_server: Rc<RefCell<APIServer>>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

fn find_percentile<T: Copy + PartialOrd>(vec: &mut [T], percentile: f64) -> T {
    let k = ((vec.len() as f64) * percentile).floor() as usize;
    *order_stat::kth_by(vec, k, |x, y| x.partial_cmp(y).unwrap())
}

impl MetricsServer {
    pub fn new(api_server: Rc<RefCell<APIServer>>, ctx: SimulationContext,
               sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            pods_resource_history: HashMap::default(),
            api_server,
            ctx,
            sim_config
        }
    }

    pub fn get_pod_snapshot_history(&self, pod_id: u64) -> Option<Vec<PodSnapshot>> {
        if self.pods_resource_history.contains_key(&pod_id) {
            let pod_history = self.pods_resource_history.get(&pod_id).unwrap();
            let mut result = Vec::with_capacity(pod_history.len());
            result.clone_from(pod_history);
            Some(result)
        } else {
            None
        }
    }

    pub fn get_pod_statistics(&self, pod_id: u64) -> Option<PodStatistic> {
        // get history of pod's resources
        let resource_history = self.get_pod_snapshot_history(pod_id);
        if resource_history.is_none() {
            return None;
        }
        let resource_history = resource_history.unwrap();

        // calculate percentiles of resources
        let mut cpu_history = Vec::with_capacity(resource_history.len());
        let mut memory_history = Vec::with_capacity(resource_history.len());
        for i in 0..resource_history.len() {
            cpu_history.push(resource_history[i].cpu);
            memory_history.push(resource_history[i].memory);
        }

        let default_lower_bound_cpu_percentile = find_percentile(&mut cpu_history, DEFAULT_LOWER_BOUND_CPU_PERCENTILE);
        let default_target_cpu_percentile = find_percentile(&mut cpu_history, DEFAULT_TARGET_CPU_PERCENTILE);
        let default_upper_bound_cpu_percentile = find_percentile(&mut cpu_history, DEFAULT_UPPER_BOUND_CPU_PERCENTILE);
        let default_lower_bound_memory_percentile = find_percentile(&mut memory_history, DEFAULT_LOWER_BOUND_MEMORY_PERCENTILE);
        let default_target_memory_percentile = find_percentile(&mut memory_history, DEFAULT_TARGET_MEMORY_PERCENTILE);
        let default_upper_memory_percentile = find_percentile(&mut memory_history, DEFAULT_UPPER_BOUND_MEMORY_PERCENTILE);

        let period_time = resource_history.last().unwrap().snapshot_time -
            resource_history.first().unwrap().snapshot_time;

        Some(PodStatistic{
            default_lower_bound_cpu_percentile,
            default_target_cpu_percentile,
            default_upper_bound_cpu_percentile,

            default_lower_bound_memory_percentile,
            default_target_memory_percentile,
            default_upper_memory_percentile,

            resource_history,
            period_time
        })
    }

    fn make_snapshot(&mut self) {
        let nodes = &self.api_server.borrow().working_nodes;
        for (_, node) in nodes.into_iter() {
            let pods = &node.borrow().pods;
            for (_, pod) in pods.into_iter() {
                let pod_snapshot = PodSnapshot {
                    cpu: pod.cpu,
                    memory: pod.memory,
                    snapshot_time: self.ctx.time()
                };
                if self.pods_resource_history.contains_key(&pod.id) {
                    self.pods_resource_history.get_mut(&pod.id).unwrap().push(pod_snapshot);
                } else {
                    let mut snapshots_vec = Vec::new();
                    snapshots_vec.push(pod_snapshot);
                    self.pods_resource_history.insert(pod.id, snapshots_vec);
                }
            }
        }
    }
}

impl EventHandler for MetricsServer {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            MetricsServerSnapshot {} => {
                self.make_snapshot();
                self.ctx.emit(MetricsServerSnapshot{}, self.id, self.sim_config.metrics_server_interval);
            }
        })
    }
}