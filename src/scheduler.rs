use std::cell::RefCell;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use crate::api_server::APIServer;
use crate::events::scheduler::SchedulerCycle;
use crate::node::{Node, NodeState};
use crate::scheduler_algorithm::SchedulerAlgorithm;
use crate::simulation_config::SimulationConfig;
use std::time::{Duration, Instant};
use crate::events::assigning::{PodAssigningFailed, PodAssigningSucceeded};
use crate::events::node::NodeStatusChanged;

pub struct Scheduler {
    pub id: u32,
    api_server: Rc<RefCell<APIServer>>,
    scheduler_algorithm: Box<dyn SchedulerAlgorithm>,
    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl Scheduler {
    pub fn new(api_server: Rc<RefCell<APIServer>>, scheduler_algorithm: Box<dyn SchedulerAlgorithm>,
               ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            api_server,
            scheduler_algorithm,
            ctx,
            sim_config
        }
    }

    pub fn schedule_next_pod(&mut self) {
        let pod_option;
        {
            pod_option = self.api_server.borrow_mut().get_pod();
        }
        match pod_option {
            None => {},
            Some(pod) => {
                let mut elapsed_time = self.sim_config.control_plane_message_delay;

                let start_of_algorithm_work = Instant::now();

                let filtered_nodes = self.scheduler_algorithm.filter(&pod,
                                                                     &self.api_server.borrow().working_nodes);
                if filtered_nodes.is_empty() {
                    elapsed_time += start_of_algorithm_work.elapsed().as_secs_f64();
                    if !self.api_server.borrow().working_nodes.is_empty() {
                        self.ctx.emit(SchedulerCycle {}, self.id, elapsed_time);
                    }
                    elapsed_time += self.sim_config.control_plane_message_delay;
                    self.ctx.emit(PodAssigningFailed { pod },
                                  self.api_server.borrow().id, elapsed_time);
                    return;
                }
                let node_scores = self.scheduler_algorithm.score(&pod,
                                                                 &self.api_server.borrow().working_nodes,
                                                                 &filtered_nodes);
                let mut max_score_ind = 0;
                for i in 0..filtered_nodes.len() {
                    if node_scores[i] > node_scores[max_score_ind] {
                        max_score_ind = i;
                    }
                }
                let node_id = filtered_nodes[max_score_ind];

                elapsed_time += start_of_algorithm_work.elapsed().as_secs_f64();
                if !self.api_server.borrow().working_nodes.is_empty() {
                    self.ctx.emit(SchedulerCycle {}, self.id, elapsed_time);
                }
                elapsed_time += self.sim_config.control_plane_message_delay;
                self.ctx.emit(PodAssigningSucceeded { pod, node_id },
                              self.api_server.borrow().id, elapsed_time);
            },
        }
    }
}

impl EventHandler for Scheduler {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            SchedulerCycle {} => {
                self.schedule_next_pod();
            }
        })
    }
}