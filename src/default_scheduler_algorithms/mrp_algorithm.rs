use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use crate::node::Node;
use crate::pod::Pod;
use crate::scheduler_algorithm::SchedulerAlgorithm;

#[derive(Default)]
pub struct MRPAlgorithm;

impl MRPAlgorithm {
    pub fn new() -> Self {
        Default::default()
    }
}

impl SchedulerAlgorithm for MRPAlgorithm {
    fn filter(&self, pod: &Pod, nodes: &BTreeMap<u32, Rc<RefCell<Node>>>) -> Vec<u32> {
        let mut filtered_nodes = Vec::<u32>::default();
        for (node_id, node) in nodes.into_iter() {
            if node.borrow().can_place_pod(pod.requested_cpu, pod.requested_memory) {
                filtered_nodes.push(*node_id);
            }
        }
        filtered_nodes
    }

    fn score(&self, pod: &Pod, nodes: &BTreeMap<u32, Rc<RefCell<Node>>>,
             filtered_node_ids: &Vec<u32>) -> Vec<f64> {
        let mut scores = Vec::<f64>::default();
        for node_id in filtered_node_ids {
            let node = nodes.get(node_id).unwrap().borrow();
            let cpu_utilization = ((node.cpu_allocated + pod.requested_cpu) as f64) / (node.cpu_total as f64);
            let memory_utilization = (node.memory_allocated + pod.requested_memory) / node.memory_total;
            scores.push(10.0 * (cpu_utilization + memory_utilization) / 2.0);
        }
        scores
    }
}