use std::cell::RefCell;
use std::collections::HashMap;
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
    fn filter(&self, pod: &Pod, nodes: &HashMap<u32, Rc<RefCell<Node>>>) -> Vec<u32> {
        let mut filtered_nodes = Vec::<u32>::default();
        for (node_id, node) in nodes.into_iter() {
            if node.borrow().get_free_cpu() >= pod.requested_cpu &&
                node.borrow().get_free_memory() >= pod.requested_memory {
                filtered_nodes.push(*node_id);
            }
        }
        filtered_nodes
    }

    fn score(&self, pod: &Pod, nodes: &HashMap<u32, Rc<RefCell<Node>>>,
             filtered_node_ids: &Vec<u32>) -> Vec<f64> {
        let mut scores = Vec::<f64>::default();
        for node_id in filtered_node_ids {
            let mut score = 10.0 * (pod.requested_cpu as f64) / (nodes.get(node_id).unwrap().borrow().get_free_cpu() as f64);
            score += 10.0 * pod.requested_memory / nodes.get(node_id).unwrap().borrow().get_free_memory();
            scores.push(score);
        }
        scores
    }
}