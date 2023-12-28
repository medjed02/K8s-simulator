use dslab_core::context::SimulationContext;
use dslab_core::simulation::Simulation;
use crate::api_server::APIServer;
use crate::scheduler::Scheduler;

pub struct K8sSimulation {
    // TODO: pointers of all objects
    scheduler: Scheduler,
    api_server: APIServer,
    
    sim: Simulation,
    ctx: SimulationContext,
}

impl K8sSimulation {
    pub fn new() {

    }

    pub fn add_host() {
        // NodeWorking to API server
    }

    pub fn remove_host() {
        // NodeFailed to API server
    }

    pub fn schedule_pod() {
        // PodAssigningRequest to API server
    }

    pub fn remove_pod() {
        // PodRemoveRequest to API server
    }


    /// Returns the average CPU load across all hosts.
    pub fn average_cpu_load(&self) -> f64 {
    }

    /// Returns the average memory load across all hosts.
    pub fn average_memory_load(&self) -> f64 {
    }

    /// Returns the current CPU allocation rate (% of overall CPU used).
    pub fn cpu_allocation_rate(&self) -> f64 {
    }

    /// Returns the current memory allocation rate (% of overall RAM used).
    pub fn memory_allocation_rate(&self) -> f64 {
    }

    /// Performs the specified number of steps through the simulation (see dslab-core docs).
    pub fn steps(&mut self, step_count: u64) -> bool {
        self.sim.steps(step_count)
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