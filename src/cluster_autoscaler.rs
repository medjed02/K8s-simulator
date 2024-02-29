use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use crate::api_server::APIServer;
use crate::cluster_autoscaler_algorithm::ClusterAutoscalerAlgorithm;
use crate::events::autoscaler::ClusterAutoscalerScan;
use crate::events::node::{AllocateNewDefaultNodes, RemoveNode};
use crate::scheduler::Scheduler;
use crate::simulation_config::SimulationConfig;

pub struct ClusterAutoscaler {
    pub id: u32,
    sim_id: u32,
    api_server: Rc<RefCell<APIServer>>,
    scheduler: Rc<RefCell<Scheduler>>,
    cluster_autoscaler_algorithm: Box<dyn ClusterAutoscalerAlgorithm>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl ClusterAutoscaler {
    pub fn new(sim_id: u32, api_server: Rc<RefCell<APIServer>>, scheduler: Rc<RefCell<Scheduler>>,
               cluster_autoscaler_algorithm: Box<dyn ClusterAutoscalerAlgorithm>,
               ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            sim_id,
            api_server,
            scheduler,
            cluster_autoscaler_algorithm,
            ctx,
            sim_config
        }
    }

    pub fn try_to_scale_up(&self) -> bool {
        let pending_pods = &self.scheduler.borrow().unschedulable_queue;
        let cnt_new_nodes = self.cluster_autoscaler_algorithm.try_to_scale_up(pending_pods,
                                                                              self.ctx.time());
        if cnt_new_nodes > 0 {
            self.ctx.emit(AllocateNewDefaultNodes { cnt_nodes: cnt_new_nodes },
                          self.sim_id, self.sim_config.default_node_allocation_time);
            true
        } else {
            false
        }
    }

    pub fn try_to_scale_down(&self) {
        let working_nodes = &self.api_server.borrow().working_nodes;
        let nodes_be_removed = self.cluster_autoscaler_algorithm.try_to_scale_down(
            working_nodes, self.ctx.time());
        for node_id in nodes_be_removed {
            self.ctx.emit(RemoveNode { node_id }, self.api_server.borrow().id,
                          self.sim_config.node_stop_duration);
        }
    }
}

impl EventHandler for ClusterAutoscaler {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            ClusterAutoscalerScan {} => {
                if !self.try_to_scale_up() {
                    self.try_to_scale_down();
                }
            }
        })
    }
}