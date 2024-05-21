use std::cell::RefCell;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, PartialEq, Clone, Copy)]
struct SimpleNode {
    pub cpu_allocated: f32,
    pub memory_allocated: f64,
    pub cpu_total: f32,
    pub memory_total: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct SimplePod {
    pub requested_cpu: f32,
    pub requested_memory: f64,
}

impl ClusterAutoscalerAlgorithm for SimpleClusterAutoscalerAlgorithm {
    fn try_to_scale_up(&mut self, pending_pods: &Vec<Pod>, now_time: f64,
                       default_node: &NodeConfig) -> u32 {
        if pending_pods.is_empty() {
            return 0;
        }
        if self.last_scale_up_time + self.scale_up_delay > now_time {
            return 0;
        }

        let mut nodes = Vec::<SimpleNode>::default();
        nodes.push(SimpleNode {
            cpu_allocated: 0., memory_allocated: 0.,
            cpu_total: default_node.cpu, memory_total: default_node.memory,
        });

        let mut pods: Vec<SimplePod> = pending_pods.iter()
            .map(
                |pod| SimplePod {
                    requested_cpu: pod.requested_cpu, requested_memory: pod.requested_memory
                })
            .collect();

        for pod in pods {
            let mut filtered_nodes: Vec<&mut SimpleNode> = nodes.iter_mut()
                .filter(
                    |node| node.cpu_allocated + pod.requested_cpu <= node.cpu_total &&
                        node.memory_allocated + pod.requested_memory <= node.memory_total
                )
                .collect();

            if filtered_nodes.is_empty() {
                drop(filtered_nodes);
                nodes.push(SimpleNode {
                    cpu_allocated: pod.requested_cpu, memory_allocated: pod.requested_memory,
                    cpu_total: default_node.cpu, memory_total: default_node.memory,
                });
            } else {
                let mut max_prior_ind = 0;
                let mut max_prior = -1.0;
                for i in 0..filtered_nodes.len() {
                    let node = &filtered_nodes[i];
                    let cpu_utilization = (node.cpu_allocated + pod.requested_cpu) / node.cpu_total;
                    let memory_utilization = (node.memory_allocated + pod.requested_memory) / node.memory_total;
                    let prior = (1.0 - cpu_utilization as f64) + (1.0 - memory_utilization);
                    if prior > max_prior {
                        max_prior = prior;
                        max_prior_ind = i;
                    }
                }
                filtered_nodes[max_prior_ind].cpu_allocated += pod.requested_cpu;
                filtered_nodes[max_prior_ind].memory_allocated += pod.requested_memory;
            }
        }

        return nodes.len() as u32;
    }

    fn try_to_scale_down(&mut self, working_nodes: &BTreeMap<u32, Rc<RefCell<Node>>>,
                         now_time: f64) -> Vec<u32> {
        let mut now_is_needed_nodes = Vec::<u32>::default();
        for (node_id, _) in &self.node_unneeded_time {
            let node = working_nodes.get(&node_id);
            if node.is_none() {
                now_is_needed_nodes.push(*node_id);
            } else if node.unwrap().borrow().cpu_allocated != 0.0 ||
                node.unwrap().borrow().memory_allocated != 0.0 {
                now_is_needed_nodes.push(*node_id);
            }
        }
        for node_id in now_is_needed_nodes {
            self.node_unneeded_time.remove(&node_id);
        }

        let mut nodes_to_scale_down = Vec::<u32>::default();
        for (node_id, node) in working_nodes {
            if node.borrow().cpu_allocated == 0.0 && node.borrow().memory_allocated == 0.0 {
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