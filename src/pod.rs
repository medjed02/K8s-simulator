//! Representation of the k8s pod

use std::fmt::{Display, Formatter};
use std::cmp::Ordering;


/// Pod status
#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone)]
pub struct Pod {
    pub id: u64,
    pub requested_cpu: f32,
    pub requested_memory: f64,
    pub limit_cpu: f32,
    pub limit_memory: f64,
    pub priority_weight: u64,
    pub status: PodStatus,
}

impl Pod {
    pub fn new(
        id: u64,
        requested_cpu: f32,
        requested_memory: f64,
        limit_cpu: f32,
        limit_memory: f64,
        priority_weight: u64,
        status: PodStatus,
    ) -> Self {
        Self {
            id,
            requested_cpu,
            requested_memory,
            limit_cpu,
            limit_memory,
            priority_weight,
            status,
        }
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