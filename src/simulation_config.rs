//! Simulation configuration.

/// Holds configuration of a single node or a set of identical nodes.
#[derive(Debug, PartialEq, Clone)]
pub struct NodeConfig {
    /// Node CPU capacity.
    pub cpu: u32,
    /// Node memory capacity in GB.
    pub memory: u64,
    /// Number of such nodes.
    pub count: u32,
}

impl NodeConfig {
    pub fn new(cpu: u32, memory: u64, count: u32) -> Self {
        Self {
            cpu,
            memory,
            count
        }
    }
}

/// Holds configuration of a single node or a set of identical pods.
#[derive(Debug, PartialEq, Clone)]
pub struct PodConfig {
    /// Minimum CPU capacity.
    pub requested_cpu: f32,
    /// Minimum memory capacity in GB.
    pub requested_memory: f64,
    /// Maximum CPU capacity.
    pub limit_cpu: f32,
    /// Maximum memory capacity in GB.
    pub limit_memory: f64,
    /// Priority weight of k8s pod (for a scheduler).
    pub priority_weight: u64,
    /// Submit time (in simulation time, seconds from start of simulation).
    pub submit_time: f64,
    /// Number of such pods.
    pub count: u32,
}

impl PodConfig {
    pub fn new(requested_cpu: f32, requested_memory: f64, limit_cpu: f32, limit_memory: f64, priority_weight: u64,
               submit_time: f64, count: u32) -> Self {
        Self {
            requested_cpu,
            requested_memory,
            limit_cpu,
            limit_memory,
            priority_weight,
            submit_time,
            count
        }
    }
}


/// Represents simulation configuration.
#[derive(Debug, PartialEq, Clone)]
pub struct SimulationConfig {
    /// Message delay in seconds for communications via network.
    pub message_delay: f64,
    /// Control plane's message delay in seconds
    pub control_plane_message_delay: f64,
    /// Pod start duration in seconds.
    pub pod_start_duration: f64,
    /// Pod stop duration in seconds.
    pub pod_stop_duration: f64,
    /// Configurations of nodes.
    pub nodes: Vec<NodeConfig>,
    /// Configurations of pods.
    pub pods: Vec<PodConfig>,
}

impl SimulationConfig {
    pub fn new(message_delay: f64, control_plane_message_delay: f64, pod_start_duration: f64,
               pod_stop_duration: f64) -> Self {
        Self {
            message_delay,
            control_plane_message_delay,
            pod_start_duration,
            pod_stop_duration,
            nodes: Vec::default(),
            pods: Vec::default(),
        }
    }
}