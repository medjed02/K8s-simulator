use std::collections::HashMap;
use serde::Serialize;
use crate::deployment::Deployment;
use crate::horizontal_autoscaler_algorithm::HorizontalAutoscalerAlgorithm;
use crate::metrics_server::PodStatistic;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ControlledResources {
    CPUOnly { cpu_utilization: Option<f32> },
    MemoryOnly { memory_utilization: Option<f64> },
    CPUAndMemory { cpu_utilization: Option<f32>, memory_utilization: Option<f64> },
}

pub struct ResourcesHorizontalAutoscalerAlgorithm {
    controlled_resources: ControlledResources,
    last_downscale_time: HashMap<u64, f64>,
    initialization_period: f64,
    time_downscale_stabilization: f64,
    min_replicas: u64,
    max_replicas: u64,
}

impl ResourcesHorizontalAutoscalerAlgorithm {
    pub fn new(controlled_resources: ControlledResources, initialization_period: f64,
               time_downscale_stabilization: f64, min_replicas: u64, max_replicas: u64) -> Self {
        Self {
            controlled_resources,
            last_downscale_time: HashMap::default(),
            initialization_period,
            time_downscale_stabilization,
            min_replicas,
            max_replicas,
        }
    }
}

impl HorizontalAutoscalerAlgorithm for ResourcesHorizontalAutoscalerAlgorithm {
    fn get_new_count_replicas(&mut self, deployment: &Deployment,
                              statistics: &Vec<PodStatistic>, now_time: f64) -> u64 {
        if self.last_downscale_time.contains_key(&deployment.id) &&
            self.last_downscale_time.get(&deployment.id).unwrap() + self.time_downscale_stabilization > now_time {
            return deployment.cnt_replicas;
        }

        let mut average_cpu = 0.0;
        let mut average_memory = 0.0;
        for statistic in statistics {
            if statistic.cpu_distribution.history_time() < self.initialization_period {
                return deployment.cnt_replicas;
            }
            average_cpu += statistic.last_snapshot.cpu;
            average_memory += statistic.last_snapshot.memory;
        }
        average_cpu /= deployment.cnt_replicas as f32;
        average_memory /= deployment.cnt_replicas as f64;

        let mut new_cnt_replicas = deployment.cnt_replicas;

        match self.controlled_resources {
            ControlledResources::CPUOnly { cpu_utilization } => {
                let target_cpu_utilization = cpu_utilization.unwrap_or(1.);
                let target_cpu = deployment.pod_template.requested_cpu * target_cpu_utilization;
                new_cnt_replicas = (average_cpu / target_cpu * deployment.cnt_replicas as f32).ceil() as u64;
            },
            ControlledResources::MemoryOnly { memory_utilization } => {
                let target_memory_utilization = memory_utilization.unwrap_or(1.);
                let target_memory = deployment.pod_template.requested_memory * target_memory_utilization;
                new_cnt_replicas = (average_memory / target_memory * deployment.cnt_replicas as f64).ceil() as u64;
            },
            ControlledResources::CPUAndMemory { cpu_utilization, memory_utilization } => {
                let target_cpu_utilization = cpu_utilization.unwrap_or(1.);
                let target_cpu = deployment.pod_template.requested_cpu * target_cpu_utilization;
                new_cnt_replicas = (average_cpu / target_cpu * deployment.cnt_replicas as f32).ceil() as u64;

                let target_memory_utilization = memory_utilization.unwrap_or(1.);
                let target_memory = deployment.pod_template.requested_memory * target_memory_utilization;
                new_cnt_replicas = new_cnt_replicas
                    .max((average_memory / target_memory * deployment.cnt_replicas as f64).ceil() as u64);
            }
        };

        if new_cnt_replicas < deployment.cnt_replicas {
            self.last_downscale_time.insert(deployment.id, now_time);
        }

        new_cnt_replicas
    }
}