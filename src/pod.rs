//! Representation of the k8s pod

use std::fmt::{Display, Formatter};
use std::cmp::Ordering;
use serde::Serialize;
use crate::deployment::Deployment;
use crate::load_model::LoadModel;


/// Pod status
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum PodStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

impl Display for PodStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PodStatus::Pending => write!(f, "pending"),
            PodStatus::Running => write!(f, "running"),
            PodStatus::Succeeded => write!(f, "succeeded"),
            PodStatus::Failed => write!(f, "failed"),
            PodStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Pod {
    pub id: u64,

    pub cpu: f32,
    pub memory: f64,
    pub cpu_load_model: Box<dyn LoadModel>,
    pub memory_load_model: Box<dyn LoadModel>,

    pub requested_cpu: f32,
    pub requested_memory: f64,
    pub limit_cpu: f32,
    pub limit_memory: f64,
    pub priority_weight: u64,

    pub scheduling_attempts: Option<u64>,
    pub scheduling_timestamp: Option<f64>,

    pub start_time: f64,
    pub status: PodStatus,

    pub deployment_id: Option<u64>,
}

impl Pod {
    pub fn new(
        id: u64,
        cpu_load_model: Box<dyn LoadModel>,
        memory_load_model: Box<dyn LoadModel>,
        requested_cpu: f32,
        requested_memory: f64,
        limit_cpu: f32,
        limit_memory: f64,
        priority_weight: u64,
        status: PodStatus,
        deployment_id: Option<u64>,
    ) -> Self {
        Self {
            id,
            cpu: 0.0,
            memory: 0.0,
            cpu_load_model,
            memory_load_model,
            requested_cpu,
            requested_memory,
            limit_cpu,
            limit_memory,
            priority_weight,
            scheduling_attempts: None,
            scheduling_timestamp: None,
            start_time: 0.0,
            status,
            deployment_id,
        }
    }
    pub fn get_wanted_cpu(&mut self, time: f64, cnt_replicas: u64) -> f64 {
        self.cpu_load_model.get_resource(time, time - self.start_time, cnt_replicas)
    }

    pub fn get_wanted_memory(&mut self, time: f64, cnt_replicas: u64) -> f64 {
        self.memory_load_model.get_resource(time, time - self.start_time, cnt_replicas)
    }
}


/// Comparision operators for prioritizing pods
impl Eq for Pod {}

impl PartialEq for Pod {
    fn eq(&self, other: &Self) -> bool {
        self.priority_weight == other.priority_weight
    }
}

impl Ord for Pod {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority_weight.cmp(&other.priority_weight)
    }
}

impl PartialOrd for Pod {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}