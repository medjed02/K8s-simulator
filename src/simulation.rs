use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use dslab_core::context::SimulationContext;
use dslab_core::{cast, Event, EventHandler};
use dslab_core::simulation::Simulation;
use crate::api_server::APIServer;
use crate::scheduler::Scheduler;
use crate::simulation_config::SimulationConfig;
use sugars::{rc, refcell};
use crate::cluster_autoscaler::ClusterAutoscaler;
use crate::cluster_autoscaler_algorithm::ClusterAutoscalerAlgorithm;
use crate::default_cluster_autoscaler_algorithms::default_simple_algorithm::SimpleClusterAutoscalerAlgorithm;
use crate::default_scheduler_algorithms::mrp_algorithm::MRPAlgorithm;
use crate::events::api_server::PodRemoveRequest;
use crate::events::assigning::PodAssigningRequest;
use crate::events::autoscaler::ClusterAutoscalerScan;
use crate::events::node::{AllocateNewDefaultNodes, NodeStatusChanged};
use crate::node::{Node, NodeState};
use crate::pod::{Pod, PodStatus};
use crate::scheduler_algorithm::SchedulerAlgorithm;

pub struct K8sSimulation {
    scheduler: Rc<RefCell<Scheduler>>,
    api_server: Rc<RefCell<APIServer>>,
    cluster_autoscaler: Rc<RefCell<ClusterAutoscaler>>,
    
    sim: Simulation,
    ctx: SimulationContext,
    sim_config: Rc<SimulationConfig>,

    last_node_id: u64,
    last_pod_id: u64,
}

impl K8sSimulation {
    /// Creates a simulation with specified config.
    pub fn new(mut sim: Simulation, sim_config: SimulationConfig,
               scheduler_algorithm: Box<dyn SchedulerAlgorithm>,
               cluster_autoscaler_algorithm: Box<dyn ClusterAutoscalerAlgorithm>,
               cluster_autoscaler_on: bool, cloud_nodes_count: u32) -> Self {
        let sim_config = rc!(sim_config);

        let api_server = rc!(refcell!(
            APIServer::new(sim.create_context("api_server"), sim_config.clone())
        ));
        sim.add_handler("api_server", api_server.clone());

        let scheduler = rc!(refcell!(Scheduler::new(api_server.clone(), scheduler_algorithm,
                sim.create_context("scheduler"), sim_config.clone())));
        sim.add_handler("scheduler", scheduler.clone());
        {
            api_server.borrow_mut().set_scheduler(scheduler.clone());
        }

        let ctx = sim.create_context("simulation");

        let mut cloud_nodes_pool = Vec::<Rc<RefCell<Node>>>::default();
        if cluster_autoscaler_on {
            for i in 0..cloud_nodes_count {
                let name = format!("cloud_node_{}", i);
                let node_ctx = sim.create_context(&name);
                let cpu = sim_config.default_node.cpu;
                let memory = sim_config.default_node.memory;
                let node = rc!(refcell!(Node::new(cpu, memory, 0.0, 0.0,
                    NodeState::Working, api_server.clone(), node_ctx, sim_config.clone())));
                cloud_nodes_pool.push(node.clone());
                sim.add_handler(name, node.clone());
            }
        }
        let cluster_ctx = sim.create_context("cluster_autoscaler");
        let cluster_autoscaler = rc!(refcell!(ClusterAutoscaler::new(
            cloud_nodes_pool, api_server.clone(), scheduler.clone(),
            cluster_autoscaler_algorithm, cluster_ctx, sim_config.clone()
        )));
        sim.add_handler("cluster_autoscaler", cluster_autoscaler.clone());

        let mut sim = Self {
            scheduler,
            api_server,
            cluster_autoscaler,
            sim,
            ctx,
            sim_config,
            last_pod_id: 0,
            last_node_id: 0
        };

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

        if cluster_autoscaler_on {
            // Launch cluster autoscaler
            sim.ctx.emit(ClusterAutoscalerScan{}, sim.cluster_autoscaler.borrow().id, 0.0);
        }

        sim
    }

    /// Add new node to the k8s cluster, return node_id
    pub fn add_node(&mut self, cpu_total: u32, memory_total: u64) -> u32 {
        self.last_node_id += 1;
        let name = format!("node_{}", self.last_node_id);
        let node_ctx = self.sim.create_context(&name);
        let node = rc!(refcell!(Node::new(cpu_total, memory_total, 0.0, 0.0, NodeState::Working,
            self.api_server.clone(), node_ctx, self.sim_config.clone())));
        let node_id = node.borrow().id;
        self.sim.add_handler(name, node.clone());
        self.api_server.borrow_mut().add_new_node(node.clone());
        node_id
    }

    pub fn recover_node(&self, node_id: u32, delay: f64) {
        self.ctx.emit(NodeStatusChanged { node_id, new_status: NodeState::Working },
                      self.api_server.borrow().id, self.sim_config.control_plane_message_delay + delay);
    }

    pub fn crash_node(&self, node_id: u32, delay: f64) {
        self.ctx.emit(NodeStatusChanged { node_id, new_status: NodeState::Failed },
                      self.api_server.borrow().id, self.sim_config.control_plane_message_delay + delay);
    }

    pub fn submit_pod(&mut self, requested_cpu: f32, requested_memory: f64, limit_cpu: f32,
                      limit_memory: f64, priority_weight: u64, delay: f64) -> u64 {
        self.last_pod_id += 1;
        let pod = Pod::new(self.last_pod_id, requested_cpu, requested_memory, limit_cpu, limit_memory,
                           priority_weight, PodStatus::Pending);
        self.ctx.emit(PodAssigningRequest { pod }, self.api_server.borrow().id,
                      self.sim_config.control_plane_message_delay + delay);
        self.last_pod_id
    }

    pub fn remove_pod(&self, pod_id: u64) {
        self.ctx.emit(PodRemoveRequest { pod_id }, self.api_server.borrow().id,
                      self.sim_config.message_delay);
    }

    /// Returns the map with references to working nodes.
    pub fn working_nodes(&self) -> BTreeMap<u32, Rc<RefCell<Node>>> {
        self.api_server.borrow().working_nodes.clone()
    }

    /// Returns the map with references to failed nodes.
    pub fn failed_nodes(&self) -> BTreeMap<u32, Rc<RefCell<Node>>> {
        self.api_server.borrow().failed_nodes.clone()
    }

    /// Returns the reference to node (node status, resources).
    pub fn node(&self, node_id: u32) -> Rc<RefCell<Node>> {
        let api_server_ref = self.api_server.borrow();
        let node = api_server_ref.working_nodes.get(&node_id);
        if node.is_some() {
            node.unwrap().clone()
        } else {
            api_server_ref.failed_nodes.get(&node_id).unwrap().clone()
        }
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