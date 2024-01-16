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