use std::cell::RefCell;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use crate::api_server::APIServer;
use crate::events::autoscaler::HorizontalAutoscalerCycle;
use crate::events::deployment::DeploymentHorizontalAutoscaling;
use crate::metrics_server::{MetricsServer, PodStatistic};
use crate::simulation_config::SimulationConfig;
use crate::horizontal_autoscaler_algorithm::HorizontalAutoscalerAlgorithm;

pub struct HorizontalAutoscaler {
    pub id: u32,
    api_server: Rc<RefCell<APIServer>>,
    metrics_server: Rc<RefCell<MetricsServer>>,
    hpa_algorithm: Box<dyn HorizontalAutoscalerAlgorithm>,

    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl HorizontalAutoscaler {
    pub fn new(api_server: Rc<RefCell<APIServer>>, metrics_server: Rc<RefCell<MetricsServer>>,
               hpa_algorithm: Box<dyn HorizontalAutoscalerAlgorithm>,
               ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            api_server,
            metrics_server,
            hpa_algorithm,
            ctx,
            sim_config
        }
    }

    pub fn try_to_scale(&mut self) {
        let api_server = self.api_server.borrow();
        let metrics_server = self.metrics_server.borrow();
        for (deployment, replicas) in &api_server.deployment_to_replicas {
            let statistics = replicas.into_iter()
                .map(|id| metrics_server.get_pod_statistics(*id));
            let not_fully_deployed = statistics.clone()
                .map(|statistic| statistic.is_none())
                .any(|x| x);
            if not_fully_deployed {
                continue
            }
            let statistics = statistics
                .map(|statistic| statistic.unwrap())
                .collect::<Vec<PodStatistic>>();
            let new_cnt_replicas = self.hpa_algorithm
                .get_new_count_replicas(deployment, &statistics, self.ctx.time());
            if new_cnt_replicas != deployment.cnt_replicas {
                self.ctx.emit(DeploymentHorizontalAutoscaling {
                    id: deployment.id, new_cnt_replicas
                }, api_server.id, self.sim_config.message_delay);
            }
        }
    }
}

impl EventHandler for HorizontalAutoscaler {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            HorizontalAutoscalerCycle {} => {
                self.try_to_scale();
                self.ctx.emit(HorizontalAutoscalerCycle{}, self.id, self.sim_config.hpa_interval);
            }
        })
    }
}