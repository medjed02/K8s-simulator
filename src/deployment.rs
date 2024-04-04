use std::hash::{Hash, Hasher};
use serde::Serialize;
use crate::load_model::LoadModel;
use crate::pod::{Pod, PodStatus};

#[derive(Clone, Serialize)]
pub struct PodTemplate {
    pub cpu_load_model: Box<dyn LoadModel>,
    pub memory_load_model: Box<dyn LoadModel>,
    pub requested_cpu: f32,
    pub requested_memory: f64,
    pub limit_cpu: f32,
    pub limit_memory: f64,
    pub priority_weight: u64,
}

#[derive(Clone, Serialize)]
pub struct Deployment {
    pub id: u64,
    pub pod_template: PodTemplate,
    pub cnt_replicas: u64,
}

impl Deployment {
    pub fn new(id: u64, pod_template: PodTemplate, cnt_replicas: u64) -> Self {
        Self {
            id, pod_template, cnt_replicas
        }
    }

    pub fn create_new_replica(&self, id: u64) -> Pod {
        Pod::new(id,
                 self.pod_template.cpu_load_model.clone(),
                 self.pod_template.memory_load_model.clone(),
                 self.pod_template.requested_cpu,
                 self.pod_template.requested_memory,
                 self.pod_template.limit_cpu,
                 self.pod_template.limit_memory,
                 self.pod_template.priority_weight,
                 PodStatus::Pending)
    }
}

impl Hash for Deployment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq<Self> for Deployment {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Deployment {}