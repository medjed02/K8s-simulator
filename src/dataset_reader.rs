use std::fs::File;
use crate::deployment::Deployment;
use crate::load_model::{ConstantLoadModel, LoadModel, ResourceSnapshot, TraceLoadModel};

#[derive(Clone)]
pub struct NodeRequest {
    pub cpu: f32,
    pub memory: f64,
}

#[derive(Clone)]
pub struct PodRequest {
    pub timestamp: f64,

    pub cpu_load_model: Box<dyn LoadModel>,
    pub memory_load_model: Box<dyn LoadModel>,

    pub requested_cpu: f32,
    pub requested_memory: f64,
    pub limit_cpu: f32,
    pub limit_memory: f64,
    pub priority_weight: u64,
}

#[derive(Clone)]
pub struct DeploymentRequest {
    pub timestamp: f64,

    pub cpu_load_model: Box<dyn LoadModel>,
    pub memory_load_model: Box<dyn LoadModel>,

    pub requested_cpu: f32,
    pub requested_memory: f64,
    pub limit_cpu: f32,
    pub limit_memory: f64,
    pub priority_weight: u64,
    pub cnt_replicas: u64,
}

#[derive(Default)]
pub struct DatasetReader {
    pub node_requests: Vec<NodeRequest>,
    pub pod_requests: Vec<PodRequest>,
    pub deployment_requests: Vec<DeploymentRequest>
}

impl DatasetReader {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse(&mut self, dataset_filename: String) {
        let file = File::open(dataset_filename).unwrap();
        let raw_json: Vec<serde_json::Value> = serde_json::from_reader(file).unwrap();
        for event in raw_json.iter() {
            if event["type"] == "ADD_NODE" {
                self.node_requests.push(NodeRequest {
                    cpu: event["cpu"].as_f64().unwrap() as f32,
                    memory: event["memory"].as_f64().unwrap(),
                })
            } else if event["type"] == "SUBMIT_POD" || event["type"] == "SUBMIT_DEPLOYMENT" {
                let requested_cpu = event["requested_cpu"].as_f64().unwrap();
                let requested_memory = event["requested_memory"].as_f64().unwrap();
                let cpu_load_model = self.parse_load_model(&event["cpu_load_model"], requested_cpu);
                let memory_load_model = self.parse_load_model(&event["memory_load_model"], requested_memory);

                if event["type"] == "SUBMIT_POD" {
                    self.pod_requests.push(PodRequest {
                        timestamp: event["timestamp"].as_f64().unwrap(),
                        cpu_load_model,
                        memory_load_model,
                        requested_cpu: requested_cpu as f32,
                        requested_memory,
                        limit_cpu: event["limit_cpu"].as_f64().unwrap() as f32,
                        limit_memory: event["limit_memory"].as_f64().unwrap(),
                        priority_weight: event["priority_weight"].as_u64().unwrap(),
                    })
                } else {
                    self.deployment_requests.push(DeploymentRequest {
                        timestamp: event["timestamp"].as_f64().unwrap(),
                        cpu_load_model,
                        memory_load_model,
                        requested_cpu: requested_cpu as f32,
                        requested_memory,
                        limit_cpu: event["limit_cpu"].as_f64().unwrap() as f32,
                        limit_memory: event["limit_memory"].as_f64().unwrap(),
                        priority_weight: event["priority_weight"].as_u64().unwrap(),
                        cnt_replicas: event["cnt_replicas"].as_u64().unwrap(),
                    })
                }
            }
        }
    }

    fn parse_load_model(&mut self, load_model_json: &serde_json::Value,
                        default_value: f64) -> Box<dyn LoadModel> {
        if load_model_json["type"] == "CONST" {
            Box::new(ConstantLoadModel::new(load_model_json["value"].as_f64().unwrap()))
        } else if load_model_json["type"] == "TRACE" {
            let mut resource_history = Vec::default();
            for snapshot in load_model_json["snapshots"].as_array().unwrap() {
                resource_history.push(ResourceSnapshot {
                    timestamp: snapshot["timestamp"].as_f64().unwrap(),
                    resource: snapshot["value"].as_f64().unwrap()
                });
            }
            Box::new(TraceLoadModel::new(resource_history))
        } else {
            Box::new(ConstantLoadModel::new(default_value))
        }
    }
}