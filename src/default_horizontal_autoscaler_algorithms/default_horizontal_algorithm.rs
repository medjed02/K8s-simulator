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
    last_scale_time: HashMap<u64, f64>,
    initialization_period: f64,
    min_replicas: u64,
    max_replicas: u64,
}

impl ResourcesHorizontalAutoscalerAlgorithm {
    pub fn new(controlled_resources: ControlledResources, initialization_period: f64,
               min_replicas: u64, max_replicas: u64) -> Self {
        Self {
            controlled_resources,
            last_scale_time: HashMap::default(),
            initialization_period,
            min_replicas,
            max_replicas,
        }
    }
}

impl HorizontalAutoscalerAlgorithm for ResourcesHorizontalAutoscalerAlgorithm {
    fn get_new_count_replicas(&mut self, deployment: &Deployment,
                              statistics: &Vec<PodStatistic>, now_time: f64) -> u64 {
        let mut new_cnt_replicas = deployment.cnt_replicas;

        let mut cpu = 0.0;
        let mut memory = 0.0;
        for statistic in statistics {
            if statistic.period_time < self.initialization_period ||
                statistic.resource_history.len() == 0 {
                return deployment.cnt_replicas;
            }
            let mut i = statistic.resource_history.len() - 1;
            let mut sum_replica_cpu = 0.0;
            let mut sum_replica_memory = 0.0;
            while i >= 0 {
                if now_time - statistic.resource_history[i].snapshot_time > self.initialization_period {
                    break
                }
                sum_replica_cpu += statistic.resource_history[i].cpu;
                sum_replica_memory += statistic.resource_history[i].memory;
                i -= 1;
            }
            let snapshots_in_window = statistic.resource_history.len() - i - 1;
            cpu += sum_replica_cpu / (snapshots_in_window as f32);
            memory += sum_replica_memory / (snapshots_in_window as f64);
        }

        match self.controlled_resources {
            ControlledResources::CPUOnly { cpu_utilization } => {
                let target_cpu_utilization = cpu_utilization.unwrap_or(1.);
                let target_cpu = deployment.pod_template.requested_cpu * target_cpu_utilization;
                new_cnt_replicas = new_cnt_replicas.max((target_cpu / cpu).ceil() as u64);
            },
            ControlledResources::MemoryOnly { memory_utilization } => {
                let target_memory_utilization = memory_utilization.unwrap_or(1.);
                let target_memory = deployment.pod_template.requested_memory * target_memory_utilization;
                new_cnt_replicas = new_cnt_replicas.max((target_memory / memory).ceil() as u64);
            },
            ControlledResources::CPUAndMemory { cpu_utilization, memory_utilization } => {
                let target_cpu_utilization = cpu_utilization.unwrap_or(1.);
                let target_cpu = deployment.pod_template.requested_cpu * target_cpu_utilization;
                new_cnt_replicas = new_cnt_replicas.max((target_cpu / cpu).ceil() as u64);

                let target_memory_utilization = memory_utilization.unwrap_or(1.);
                let target_memory = deployment.pod_template.requested_memory * target_memory_utilization;
                new_cnt_replicas = new_cnt_replicas.max((target_memory / memory).ceil() as u64);
            }
        };

        new_cnt_replicas
    }
}