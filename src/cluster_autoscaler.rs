use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, Simulation, SimulationContext};
use sugars::{rc, refcell};
use crate::api_server::APIServer;
use crate::cluster_autoscaler_algorithm::ClusterAutoscalerAlgorithm;
use crate::events::autoscaler::ClusterAutoscalerScan;
use crate::events::node::{AllocateNewDefaultNodes, RemoveNode};
use crate::node::{Node, NodeState};
use crate::scheduler::Scheduler;
use crate::simulation_config::SimulationConfig;

pub struct ClusterAutoscaler {
    pub id: u32,
    cloud_nodes_pool: Vec<Rc<RefCell<Node>>>,
    api_server: Rc<RefCell<APIServer>>,
    scheduler: Rc<RefCell<Scheduler>>,
    cluster_autoscaler_algorithm: Box<dyn ClusterAutoscalerAlgorithm>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl ClusterAutoscaler {
    pub fn new(cloud_nodes_pool: Vec<Rc<RefCell<Node>>>, api_server: Rc<RefCell<APIServer>>,
               scheduler: Rc<RefCell<Scheduler>>,
               cluster_autoscaler_algorithm: Box<dyn ClusterAutoscalerAlgorithm>,
               ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            cloud_nodes_pool,
            api_server,
            scheduler,
            cluster_autoscaler_algorithm,
            ctx,
            sim_config
        }
    }

    pub fn try_to_scale_up(&mut self) -> bool {
        let pending_pods = &self.scheduler.borrow().unschedulable_queue;
        let cnt_new_nodes = self.cluster_autoscaler_algorithm.try_to_scale_up(pending_pods,
                                                                              self.ctx.time(),
                                                                              &self.sim_config.default_node);
        if cnt_new_nodes > 0 {
            self.ctx.emit(AllocateNewDefaultNodes { cnt_nodes: cnt_new_nodes },
                          self.id, self.sim_config.default_node_allocation_time);
            true
        } else {
            false
        }
    }

    pub fn try_to_scale_down(&mut self) {
        let working_nodes = &self.api_server.borrow().working_nodes;
        let nodes_be_removed = self.cluster_autoscaler_algorithm.try_to_scale_down(
            working_nodes, self.ctx.time());
        for node_id in nodes_be_removed {
            self.ctx.emit(RemoveNode { node_id }, self.api_server.borrow().id,
                          self.sim_config.node_stop_duration);
        }
    }

    /// Allocate new node for scale up (from cloud pool)
    pub fn allocate_new_node(&mut self) {
        let node = self.cloud_nodes_pool.pop().unwrap();
        self.api_server.borrow_mut().add_new_node(node);
    }
}

impl EventHandler for ClusterAutoscaler {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            ClusterAutoscalerScan {} => {
                if !self.try_to_scale_up() {
                    self.try_to_scale_down();
                }
                self.ctx.emit(ClusterAutoscalerScan{}, self.id,
                    self.sim_config.cluster_autoscaler_scan_interval);
            }
            AllocateNewDefaultNodes { cnt_nodes } => {
                for _ in 0..cnt_nodes.min(self.cloud_nodes_pool.len() as u32) {
                    self.allocate_new_node();
                }
            }
        })
    }
}