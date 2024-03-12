use std::cell::RefCell;
use std::rc::Rc;
use crate::metrics_server::PodSnapshot;
use crate::node::Node;
use crate::pod::Pod;

#[derive(Copy, Clone)]
pub struct VPARecommendation {
    pub pod_id: u64,
    pub new_requested_cpu: f32,
    pub new_limit_cpu: f32,
    pub new_requested_memory: f64,
    pub new_limit_memory: f64,
}

pub trait VerticalAutoscalerAlgorithm {
    fn get_recommendation(&mut self, pod_id: u64,
                          pod_snapshot_history: Vec<PodSnapshot>) -> Option<VPARecommendation>;

    fn try_to_apply_recommendation(&mut self, pod: &Pod, node: &Rc<RefCell<Node>>,
                                   recommendation: VPARecommendation) -> bool;
}