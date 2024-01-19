use std::cell::RefCell;
use std::rc::Rc;
use dslab_core::context::SimulationContext;
use dslab_core::simulation::Simulation;
use crate::api_server::APIServer;
use crate::scheduler::Scheduler;
use crate::simulation_config::SimulationConfig;
use sugars::{rc, refcell};
use crate::default_scheduler_algorithms::mrp_algorithm::MRPAlgorithm;
use crate::events::assigning::PodAssigningRequest;
use crate::events::node::NodeStatusChanged;
use crate::node::{Node, NodeState};
use crate::pod::{Pod, PodStatus};

pub struct K8sSimulation {
    scheduler: Rc<RefCell<Scheduler>>,
    api_server: Rc<RefCell<APIServer>>,
    
    sim: Simulation,
    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,

    last_node_id: u64,
    last_pod_id: u64,
}

impl K8sSimulation {
    /// Creates a simulation with specified config.
    pub fn new(mut sim: Simulation, sim_config: SimulationConfig) -> Self {
        let sim_config = rc!(sim_config);

        let api_server = rc!(refcell!(
            APIServer::new(sim.create_context("api_server"), sim_config.clone())
        ));
        sim.add_handler("api_server", api_server.clone());

        let ctx = sim.create_context("simulation");
        let mut sim = Self {
            scheduler: rc!(refcell!(Scheduler::new(api_server.clone(), Box::new(MRPAlgorithm::new()),
                sim.create_context("scheduler"), sim_config.clone()))),
            api_server,
            sim,
            ctx,
            sim_config,
            last_pod_id: 0,
            last_node_id: 0
        };

        sim.sim.add_handler("scheduler", sim.scheduler.clone());
        {
            sim.api_server.borrow_mut().set_scheduler(sim.scheduler.clone());
        }

        for node_config in sim.sim_config.nodes.clone() {
            for i in 0..node_config.count {
                sim.add_node(node_config.cpu, node_config.memory);
            }
        }

        for pod_config in sim.sim_config.pods.clone() {
            for i in 0..pod_config.count {
                sim.submit_pod(pod_config.requested_cpu, pod_config.requested_memory, pod_config.limit_cpu,
                               pod_config.limit_memory, pod_config.priority_weight, pod_config.submit_time);
            }
        }

        sim
    }

    /// Add new node to the k8s cluster, return node_id
    pub fn add_node(&mut self, cpu_total: u32, memory_total: u64) -> u64 {
        self.last_node_id += 1;
        let name = format!("node_{}", self.last_node_id);
        let node_ctx = self.sim.create_context(&name);
        let node = rc!(refcell!(Node::new(cpu_total, memory_total, 0.0, 0.0, NodeState::Working,
            self.api_server.clone(), node_ctx, self.sim_config.clone())));
        self.sim.add_handler(name, node.clone());
        self.api_server.borrow_mut().add_new_node(node.clone());
        self.last_node_id
    }

    pub fn recover_node(&self, node_id: u32, delay: f64) {
        self.ctx.emit(NodeStatusChanged { node_id, new_status: NodeState::Working },
                      self.sim.lookup_id("api_server"), self.sim_config.control_plane_message_delay + delay);
    }

    pub fn remove_node(&self, node_id: u32, delay: f64) {
        self.ctx.emit(NodeStatusChanged { node_id, new_status: NodeState::Failed },
                      self.sim.lookup_id("api_server"), self.sim_config.control_plane_message_delay + delay);
    }

    pub fn submit_pod(&mut self, requested_cpu: f32, requested_memory: f64, limit_cpu: f32,
                      limit_memory: f64, priority_weight: u64, delay: f64) -> u64 {
        self.last_pod_id += 1;
        let pod = Pod::new(self.last_pod_id, requested_cpu, requested_memory, limit_cpu, limit_memory,
                           priority_weight, PodStatus::Pending);
        self.ctx.emit(PodAssigningRequest { pod }, self.sim.lookup_id("api_server"),
                      self.sim_config.control_plane_message_delay + delay);
        self.last_pod_id
    }

    pub fn remove_pod() {
        // PodRemoveRequest to API server
    }


    /// Returns the average CPU load across all working nodes.
    pub fn average_cpu_load(&self) -> f64 {
        self.api_server.borrow().average_cpu_load()
    }

    /// Returns the average memory load across all working nodes.
    pub fn average_memory_load(&self) -> f64 {
        self.api_server.borrow().average_memory_load()
    }

    /// Returns the current CPU load rate (% of overall CPU used).
    pub fn cpu_load_rate(&self) -> f64 {
        self.api_server.borrow().average_cpu_load()
    }

    /// Returns the current memory load rate (% of overall RAM used).
    pub fn memory_load_rate(&self) -> f64 {
        self.api_server.borrow().memory_load_rate()
    }

    /// Performs the specified number of steps through the simulation (see dslab-core docs).
    pub fn steps(&mut self, step_count: u64) -> bool {
        self.sim.steps(step_count)
    }

    /// Steps through the simulation until there are no pending events left.
    pub fn step_until_no_events(&mut self) {
        self.sim.step_until_no_events();
    }

    /// Steps through the simulation with duration limit (see dslab-core docs).
    pub fn step_for_duration(&mut self, time: f64) {
        self.sim.step_for_duration(time);
    }

    /// Steps through the simulation until the specified time (see dslab-core docs).
    pub fn step_until_time(&mut self, time: f64) {
        self.sim.step_until_time(time);
    }

    /// Returns the total number of created events.
    pub fn event_count(&self) -> u64 {
        self.sim.event_count()
    }

    /// Returns the current simulation time.
    pub fn current_time(&self) -> f64 {
        self.sim.time()
    }
}