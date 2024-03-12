use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
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

#[derive(Copy, Clone)]
pub struct PodStatistic {
    pub cpu_percentile: f32,
    pub memory_percentile: f64,
    pub period_time: f64,
}

pub struct MetricsServer {
    pub id: u32,
    pods_resource_history: HashMap<u64, Vec<PodSnapshot>>,
    api_server: Rc<RefCell<APIServer>>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
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

    pub fn get_pod_statistics(&self, pod_id: u64, percentile: f64) -> Option<PodStatistic> {
        let resource_history = self.pods_resource_history.get(&pod_id);
        if resource_history.is_none() {
            return None;
        }
        let resource_history = resource_history.unwrap();
        let mut cpu_history = Vec::with_capacity(resource_history.len());
        let mut memory_history = Vec::with_capacity(resource_history.len());
        for i in 0..resource_history.len() {
            cpu_history.push_back(resource_history[i].cpu);
            memory_history.push_back(resource_history[i].memory);
        }
        let k = ((resource_history.len() as f64) * percentile).floor() as usize;
        let cpu_percentile = *order_stat::kth_by(&mut cpu_history, k,
                                                 |x, y| x.partial_cmp(y).unwrap());
        let memory_percentile = *order_stat::kth_by(&mut memory_history, k,
                                                    |x, y| x.partial_cmp(y).unwrap());
        let period_time = resource_history.back().unwrap().snapshot_time -
            resource_history.front().unwrap().snapshot_time;
        Some(PodStatistic{ cpu_percentile, memory_percentile, period_time })
    }

    fn make_snapshot(&mut self) {
        let nodes = &self.api_server.borrow().working_nodes;
        for (_, node) in nodes.into_iter() {
            let pods = &node.borrow().pods;
            for (_, pod) in pods.into_iter() {
                let pod_snapshot = PodSnapshot {
                    cpu: *pod.cpu,
                    memory: *pod.memory,
                    snapshot_time: self.ctx.time()
                };
                if self.pods_resource_history.contains_key(*pod.id) {
                    self.pods_resource_history.get_mut(*pod.id).unwrap().push_back(pod_snapshot);
                } else {
                    let mut snapshots_vec = VecDeque::new();
                    snapshots_vec.push_back(pod_snapshot);
                    self.pods_resource_history.insert(*pod.id, snapshots_vec);
                }
            }
        }
    }

    fn compress_statistic(&mut self) {

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