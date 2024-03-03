use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::rc::Rc;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use crate::api_server::APIServer;
use crate::events::scheduler::{FlushUnschedulableQueue, MoveRequest, PodBackoffRetry, SchedulingCycle};
use crate::node::{Node, NodeState};
use crate::scheduler_algorithm::SchedulerAlgorithm;
use crate::simulation_config::SimulationConfig;
use std::time::{Duration, Instant};
use crate::events::assigning::{PodAssigningFailed, PodAssigningSucceeded};
use crate::events::node::NodeStatusChanged;
use crate::pod::Pod;

const UNSCHEDULABLE_QUEUE_FLUSH_TIMEOUT: f64 = 30.0;
const POD_MIN_UNSCHEDULABLE_TIMEOUT: f64 = 30.0;

pub struct Scheduler {
    pub id: u32,
    pub active_queue: BinaryHeap<Pod>,
    pub unschedulable_queue: Vec<Pod>,
    api_server: Rc<RefCell<APIServer>>,
    scheduler_algorithm: Box<dyn SchedulerAlgorithm>,
    scheduling_cycle: i64,
    moving_cycle: i64,
    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,
}

impl Scheduler {
    pub fn new(api_server: Rc<RefCell<APIServer>>, scheduler_algorithm: Box<dyn SchedulerAlgorithm>,
               ctx: SimulationContext, sim_config: Rc<SimulationConfig>) -> Self {
        Self {
            id: ctx.id(),
            active_queue: BinaryHeap::default(),
            unschedulable_queue: Vec::default(),
            api_server,
            scheduler_algorithm,
            scheduling_cycle: 0,
            moving_cycle: -1,
            ctx,
            sim_config
        }
    }

    /// Add pod to the ActiveQueue
    pub fn add_pod(&mut self, pod: Pod) {
        if self.active_queue.is_empty() {
            self.ctx.emit(SchedulingCycle {}, self.id, 0.0);
        }
        self.active_queue.push(pod);
    }

    /// Pop next pod in the ActiveQueue
    pub fn get_pod(&mut self) -> Option<Pod> {
        self.active_queue.pop()
    }

    pub fn schedule_next_pod(&mut self) {
        match self.get_pod() {
            None => {},
            Some(mut pod) => {
                self.scheduling_cycle += 1;

                if pod.scheduling_attempts.is_some() {
                    pod.scheduling_attempts = Some(pod.scheduling_attempts.unwrap() + 1);
                } else {
                    pod.scheduling_attempts = Some(0);
                }

                let mut elapsed_time = self.sim_config.control_plane_message_delay;

                let start_of_algorithm_work = Instant::now();

                let filtered_nodes = self.scheduler_algorithm.filter(&pod,
                                                                     &self.api_server.borrow().working_nodes);
                if filtered_nodes.is_empty() {
                    elapsed_time += start_of_algorithm_work.elapsed().as_secs_f64();
                    if !self.api_server.borrow().working_nodes.is_empty() {
                        self.ctx.emit(SchedulingCycle {}, self.id, elapsed_time);
                    }
                    elapsed_time += self.sim_config.control_plane_message_delay;
                    self.ctx.emit(PodAssigningFailed { pod, scheduling_cycle: self.scheduling_cycle },
                                  self.id, elapsed_time);
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
                    self.ctx.emit(SchedulingCycle {}, self.id, elapsed_time);
                }
                elapsed_time += self.sim_config.control_plane_message_delay;
                pod.scheduling_attempts = None;
                pod.scheduling_timestamp = None;
                self.ctx.emit(PodAssigningSucceeded { pod, node_id },
                              self.api_server.borrow().id, elapsed_time);
            },
        }
    }

    pub fn add_pod_to_unschedulable(&mut self, mut pod: Pod, scheduling_cycle: i64) {
        if self.moving_cycle < scheduling_cycle {
            pod.scheduling_timestamp = Some(self.ctx.time());
            self.unschedulable_queue.push(pod);
        } else {
            let backoff_duration = self.calculate_backoff_duration(&pod);
            self.ctx.emit(PodBackoffRetry { pod }, self.id, backoff_duration);
        }
    }

    fn calculate_backoff_duration(&self, pod: &Pod) -> f64 {
        let mut duration = self.sim_config.pod_initial_backoff_duration;
        for i in 0..pod.scheduling_attempts.unwrap() {
            duration *= 2.0;
            if duration >= self.sim_config.pod_max_backoff_duration {
                return duration;
            }
        }
        duration
    }

    pub fn move_pods_to_active_or_backoff(&mut self, mut pods: Vec<Pod>) {
        while !pods.is_empty() {
            let pod = pods.pop().unwrap();
            if self.ctx.time() - pod.scheduling_timestamp.unwrap() < self.calculate_backoff_duration(&pod) {
                let left_backoff_duration = self.calculate_backoff_duration(&pod) - self.ctx.time() + pod.scheduling_timestamp.unwrap();
                self.ctx.emit(PodBackoffRetry { pod }, self.id, left_backoff_duration);
            } else {
                self.add_pod(pod);
            }
        }
        self.moving_cycle = self.scheduling_cycle;
    }

    pub fn flush_unschedulable_queue(&mut self) {
        let mut new_unschedulable_queue = Vec::<Pod>::default();
        let mut pods_to_flush = Vec::<Pod>::default();
        while !self.unschedulable_queue.is_empty() {
            let pod = self.unschedulable_queue.pop().unwrap();
            if self.ctx.time() - pod.scheduling_timestamp.unwrap() < POD_MIN_UNSCHEDULABLE_TIMEOUT {
                new_unschedulable_queue.push(pod);
            } else {
                pods_to_flush.push(pod);
            }
        }
        self.unschedulable_queue = new_unschedulable_queue;
        self.move_pods_to_active_or_backoff(pods_to_flush);
    }

    pub fn move_all_to_active_or_backoff(&mut self) {
        let mut pods_to_move = Vec::<Pod>::default();
        while !self.unschedulable_queue.is_empty() {
            let pod = self.unschedulable_queue.pop().unwrap();
            pods_to_move.push(pod);
        }
        self.move_pods_to_active_or_backoff(pods_to_move);
    }

}

impl EventHandler for Scheduler {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            SchedulingCycle {} => {
                self.schedule_next_pod();
            }
            PodAssigningFailed { pod, scheduling_cycle } => {
                self.add_pod_to_unschedulable(pod, scheduling_cycle);
            }
            PodBackoffRetry { pod } => {
                self.add_pod(pod);
            }
            FlushUnschedulableQueue {} => {
                self.ctx.emit(FlushUnschedulableQueue {}, self.id, UNSCHEDULABLE_QUEUE_FLUSH_TIMEOUT);
                self.flush_unschedulable_queue();
            }
            MoveRequest {} => {
                self.move_all_to_active_or_backoff();
            }
        })
    }
}