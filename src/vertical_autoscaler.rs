use std::cell::RefCell;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use crate::api_server::APIServer;
use crate::events::autoscaler::VerticalAutoscalerCycle;
use crate::events::pod::PodRequestAndLimitsChange;
use crate::metrics_server::MetricsServer;
use crate::simulation_config::SimulationConfig;
use crate::vertical_autoscaler_algorithm::{VerticalAutoscalerAlgorithm, VPARecommendation};

pub struct VerticalAutoscaler {
    pub id: u32,
    api_server: Rc<RefCell<APIServer>>,
    metrics_server: Rc<RefCell<MetricsServer>>,
    vpa_algorithm: Box<dyn VerticalAutoscalerAlgorithm>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl VerticalAutoscaler {
    pub fn new(api_server: Rc<RefCell<APIServer>>, metrics_server: Rc<RefCell<MetricsServer>>,
               vpa_algorithm: Box<dyn VerticalAutoscalerAlgorithm>,
               ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            api_server,
            metrics_server,
            vpa_algorithm,
            ctx,
            sim_config
        }
    }

    fn collect_recommendations(&mut self) -> Vec<VPARecommendation> {
        let mut recommendations = Vec::<VPARecommendation>::default();
        let pod_ids = self.api_server.borrow().pod_to_node_map.keys();
        for pod_id in pod_ids {
            let snapshot_history = self.metrics_server.borrow().get_pod_snapshot_history(*pod_id);
            if snapshot_history.is_none() {
                continue;
            }
            let snapshot_history = snapshot_history.unwrap();
            let recommendation = self.vpa_algorithm.get_recommendation(*pod_id, snapshot_history);
            if recommendation.is_some() {
                recommendations.push(recommendation.unwrap());
            }
        }
        recommendations
    }

    fn try_to_apply_recommendations(&mut self, recommendations: Vec<VPARecommendation>) {
        for recommendation in recommendations {
            let node_id = self.api_server.borrow().pod_to_node_map.get(&recommendation.pod_id);
            if node_id.is_none() {
                continue;
            }
            let node_id = node_id.unwrap();
            let node = self.api_server.borrow().working_nodes.get(&node_id);
            if node.is_none() {
                continue;
            }
            let node = node.unwrap();
            let pod = node.borrow().pods.get(&recommendation.pod_id);
            if pod.is_none() {
                continue;
            }
            let pod = pod.unwrap();
            if self.vpa_algorithm.try_to_apply_recommendation(pod, node, recommendation) {
                self.ctx.emit(PodRequestAndLimitsChange {
                    pod_id: recommendation.pod_id,
                    new_requested_cpu: recommendation.new_requested_cpu,
                    new_limit_cpu: recommendation.new_limit_cpu,
                    new_requested_memory: recommendation.new_requested_memory,
                    new_limit_memory: recommendation.new_limit_memory
                }, *node_id, self.sim_config.message_delay * 2);
            }
        }
    }
}

impl EventHandler for VerticalAutoscaler {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            VerticalAutoscalerCycle {} => {
                let recommendations = self.collect_recommendations();
                self.try_to_apply_recommendations(recommendations);
                self.ctx.emit(VerticalAutoscalerCycle{}, self.id, self.sim_config.vpa_interval);
            }
        })
    }
}