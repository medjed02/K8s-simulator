use crate::deployment::Deployment;
use crate::metrics_server::PodStatistic;

pub trait HorizontalAutoscalerAlgorithm {
    fn get_new_count_replicas(&mut self, deployment: &Deployment,
                              statistics: &Vec<PodStatistic>, now_time: f64) -> u64;
}