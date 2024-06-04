use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use crate::api_server::APIServer;
use crate::events::autoscaler::MetricsServerSnapshot;
use crate::histogram::Histogram;
use crate::simulation_config::SimulationConfig;

#[derive(Copy, Clone)]
pub struct PodSnapshot {
    pub cpu: f32,
    pub memory: f64,
    pub snapshot_time: f64,
}

#[derive(Clone)]
pub struct PodStatistic<'a> {
    pub cpu_distribution: &'a Histogram,
    pub memory_distribution: &'a Histogram,
    pub last_snapshot: PodSnapshot,
}

pub struct MetricsServer {
    pub id: u32,
    pods_cpu_distribution: HashMap<u64, Histogram>,
    pods_memory_distribution: HashMap<u64, Histogram>,
    pods_last_snapshot: HashMap<u64, PodSnapshot>,

    api_server: Rc<RefCell<APIServer>>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl MetricsServer {
    pub fn new(api_server: Rc<RefCell<APIServer>>, ctx: SimulationContext,
               sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            pods_cpu_distribution: HashMap::default(),
            pods_memory_distribution: HashMap::default(),
            pods_last_snapshot: HashMap::default(),
            api_server,
            ctx,
            sim_config
        }
    }

    pub fn get_pod_statistics(&self, pod_id: u64) -> Option<PodStatistic> {
        // get last snapshot
        let last_snapshot = self.pods_last_snapshot.get(&pod_id);
        if last_snapshot.is_none() {
            return None;
        }
        let last_snapshot = *last_snapshot.unwrap();

        Some(PodStatistic {
            cpu_distribution: &self.pods_cpu_distribution.get(&pod_id).unwrap(),
            memory_distribution: &self.pods_memory_distribution.get(&pod_id).unwrap(),
            last_snapshot,
        })
    }

    pub fn clear_pod_statistics(&mut self, pod_id: u64) {
        self.pods_cpu_distribution.remove(&pod_id);
        self.pods_memory_distribution.remove(&pod_id);
        self.pods_last_snapshot.remove(&pod_id);
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
                if !self.pods_cpu_distribution.contains_key(&pod.id) {
                    self.pods_cpu_distribution.insert(pod.id, Histogram::new(pod.limit_cpu as f64));
                    self.pods_memory_distribution.insert(pod.id, Histogram::new(pod.limit_memory));
                }
                self.pods_cpu_distribution.get_mut(&pod.id).unwrap()
                    .add_sample(pod_snapshot.cpu as f64, 1, self.ctx.time());
                self.pods_memory_distribution.get_mut(&pod.id).unwrap()
                    .add_sample(pod_snapshot.memory, 1, self.ctx.time());
                self.pods_last_snapshot.insert(pod.id, pod_snapshot);
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