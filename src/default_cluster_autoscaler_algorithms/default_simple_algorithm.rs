use std::cell::RefCell;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use crate::cluster_autoscaler_algorithm::ClusterAutoscalerAlgorithm;
use crate::node::Node;
use crate::pod::Pod;
use crate::simulation_config::NodeConfig;

#[derive(Default)]
pub struct SimpleClusterAutoscalerAlgorithm {
    node_unneeded_time: HashMap<u32, f64>,
    scale_down_unneeded_time: f64,
    max_empty_bulk_delete: u32,
    last_scale_up_time: f64,
    last_scale_down_time: f64,
    scale_up_delay: f64,
}

impl SimpleClusterAutoscalerAlgorithm {
    pub fn new(scale_down_unneeded_time: f64, max_empty_bulk_delete: u32, scale_up_delay: f64) -> Self {
        Self {
            node_unneeded_time: HashMap::default(),
            scale_down_unneeded_time,
            max_empty_bulk_delete,
            last_scale_up_time: 0.0,
            last_scale_down_time: 0.0,
            scale_up_delay
        }
    }
}

impl ClusterAutoscalerAlgorithm for SimpleClusterAutoscalerAlgorithm {
    fn try_to_scale_up(&mut self, pending_pods: &Vec<Pod>, now_time: f64,
                       default_node: &NodeConfig) -> u32 {
        if pending_pods.is_empty() {
            return 0;
        }
        if self.last_scale_up_time + self.last_scale_up_time > now_time {
            return 0;
        }
        let mut sum_cpu = 0.0;
        let mut sum_memory = 0.0;
        for pod in pending_pods {
            sum_cpu += pod.requested_cpu;
            sum_memory += pod.requested_memory;
        }
        let need_default_nodes_cpu = (sum_cpu / (default_node.cpu as f32)).ceil() as u32;
        let need_default_nodes_memory = (sum_memory / (default_node.memory as f64)).ceil() as u32;
        self.last_scale_up_time = now_time;
        return max(need_default_nodes_cpu, need_default_nodes_memory);
    }

    fn try_to_scale_down(&mut self, working_nodes: &BTreeMap<u32, Rc<RefCell<Node>>>,
                         now_time: f64) -> Vec<u32> {
        let mut now_is_needed_nodes = Vec::<u32>::default();
        for (node_id, _) in &self.node_unneeded_time {
            let node = working_nodes.get(&node_id);
            if node.is_none() {
                now_is_needed_nodes.push(*node_id);
            } else if node.unwrap().borrow().cpu_load != 0.0 ||
                node.unwrap().borrow().memory_load != 0.0 {
                now_is_needed_nodes.push(*node_id);
            }
        }
        for node_id in now_is_needed_nodes {
            self.node_unneeded_time.remove(&node_id);
        }

        let mut nodes_to_scale_down = Vec::<u32>::default();
        for (node_id, node) in working_nodes {
            if node.borrow().cpu_load == 0.0 && node.borrow().memory_load == 0.0 {
                let node_in_unneeded = self.node_unneeded_time.get(node_id);
                if node_in_unneeded.is_none() {
                    self.node_unneeded_time.insert(*node_id, now_time);
                } else if now_time - node_in_unneeded.unwrap() >= self.scale_down_unneeded_time {
                    nodes_to_scale_down.push(*node_id);
                    if nodes_to_scale_down.len() == (self.max_empty_bulk_delete as usize) {
                        break;
                    }
                }
            }
        }
        if !nodes_to_scale_down.is_empty() {
            for node_id in &nodes_to_scale_down {
                self.node_unneeded_time.remove(node_id);
            }
        }
        self.last_scale_down_time = now_time;
        nodes_to_scale_down
    }
}