use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use crate::node::Node;
use crate::pod::Pod;
use crate::simulation_config::NodeConfig;

pub trait ClusterAutoscalerAlgorithm {
    /// Checks the need for scaling up, returns the number of new nodes
    fn try_to_scale_up(&mut self, pending_pods: &Vec<Pod>, now_time: f64, default_node: &NodeConfig) -> u32;

    /// Checks the need for scaling down, returns ids of nodes to be deleted
    fn try_to_scale_down(&mut self, working_nodes: &BTreeMap<u32, Rc<RefCell<Node>>>,
                         now_time: f64) -> Vec<u32>;
}