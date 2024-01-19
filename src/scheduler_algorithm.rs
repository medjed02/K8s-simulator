use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::node::Node;
use crate::pod::Pod;

pub trait SchedulerAlgorithm {
    /// Filter nodes by need pod, returns node_id for filtered nodes.
    fn filter(&self, pod: &Pod, nodes: &HashMap<u32, Rc<RefCell<Node>>>) -> Vec<u32>;

    /// Score nodes by need pod, returns scores for nodes from filtered_node_ids.
    fn score(&self, pod: &Pod, nodes: &HashMap<u32, Rc<RefCell<Node>>>,
             filtered_node_ids: &Vec<u32>) -> Vec<f64>;
}

//pub fn scheduler_algorithm_resolver(config_str: String) -> Box<dyn SchedulerAlgorithm> {
//}