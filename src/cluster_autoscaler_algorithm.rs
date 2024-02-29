use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use crate::node::Node;
use crate::pod::Pod;

pub trait ClusterAutoscalerAlgorithm {
    /// Checks the need for scaling up, returns the number of new nodes
    fn try_to_scale_up(&self, pending_pods: &Vec<Pod>, now_time: f64) -> u32;

    /// Checks the need for scaling down, returns ids of nodes to be deleted
    fn try_to_scale_down(&self, working_nodes: &BTreeMap<u32, Rc<RefCell<Node>>>,
                         now_time: f64) -> Vec<u32>;
}