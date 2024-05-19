//! Simulation configuration.

use serde::{Deserialize, Serialize};

/// Holds configuration of a single node or a set of identical nodes.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node CPU capacity.
    pub cpu: f32,
    /// Node memory capacity in GB.
    pub memory: f64,
    /// Number of such nodes.
    pub count: u32,
}

impl NodeConfig {
    pub fn new(cpu: f32, memory: f64, count: u32) -> Self {
        Self {
            cpu,
            memory,
            count
        }
    }
}

/// Holds configuration of a single node or a set of identical pods.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

/// Holds information about the used trace dataset.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DatasetConfig {
    /// Dataset path.
    pub path: String,
}

/// Holds raw simulation config parsed from YAML file.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct RawSimulationConfig {
    pub message_delay: Option<f64>,
    pub control_plane_message_delay: Option<f64>,
    pub pod_start_duration: Option<f64>,
    pub pod_stop_duration: Option<f64>,
    pub node_stop_duration: Option<f64>,
    pub pod_initial_backoff_duration: Option<f64>,
    pub pod_max_backoff_duration: Option<f64>,
    pub cluster_autoscaler_scan_interval: Option<f64>,
    pub default_node: Option<NodeConfig>,
    pub default_node_allocation_time: Option<f64>,
    pub cloud_nodes_count: Option<u64>,
    pub metrics_server_interval: Option<f64>,
    pub vpa_interval: Option<f64>,
    pub hpa_interval: Option<f64>,
    pub memory_pressure_threshold: Option<f64>,
    pub update_pods_resources_period: Option<f64>,
    pub nodes: Option<Vec<NodeConfig>>,
    pub trace: Option<DatasetConfig>,
}

/// Represents simulation configuration.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Message delay in seconds for communications via network.
    pub message_delay: f64,
    /// Control plane's message delay in seconds
    pub control_plane_message_delay: f64,
    /// Pod start duration in seconds.
    pub pod_start_duration: f64,
    /// Pod stop duration in seconds.
    pub pod_stop_duration: f64,
    /// Node stop duration in seconds
    pub node_stop_duration: f64,
    /// Initial backoff duration for scheduler
    pub pod_initial_backoff_duration: f64,
    /// Max backoff duration for scheduler
    pub pod_max_backoff_duration: f64,
    /// Scan interval for cluster autoscaler in seconds
    pub cluster_autoscaler_scan_interval: f64,
    /// Configuration of default pod (for cluster autoscaler)
    pub default_node: NodeConfig,
    /// Time of allocation of new node (for cluster autoscaler)
    pub default_node_allocation_time: f64,
    /// Count of default nodes in cloud pool (for cluster autoscaler)
    pub cloud_nodes_count: u64,
    /// Time of metrics server's interval
    pub metrics_server_interval: f64,
    /// Time of VPA interval
    pub vpa_interval: f64,
    /// Time of HPA interval
    pub hpa_interval: f64,
    pub memory_pressure_threshold: f64,
    pub update_pods_resources_period: f64,
    /// Configurations of nodes.
    pub nodes: Vec<NodeConfig>,
    /// Used trace dataset.
    pub trace: Option<DatasetConfig>,
}

impl SimulationConfig {
    pub fn new(message_delay: f64, control_plane_message_delay: f64, pod_start_duration: f64,
               pod_stop_duration: f64, pod_initial_backoff_duration: f64,
               pod_max_backoff_duration: f64) -> Self {
        Self {
            message_delay,
            control_plane_message_delay,
            pod_start_duration,
            pod_stop_duration,
            node_stop_duration: 30.0,
            pod_initial_backoff_duration,
            pod_max_backoff_duration,
            cluster_autoscaler_scan_interval: 10.0,
            default_node: NodeConfig::new(8., 64., 1),
            default_node_allocation_time: 120.0,
            cloud_nodes_count: 100,
            metrics_server_interval: 30.0,
            vpa_interval: 30.0,
            hpa_interval: 30.0,
            memory_pressure_threshold: 0.95,
            update_pods_resources_period: 10.0,
            nodes: Vec::default(),
            trace: None,
        }
    }

    pub fn from_file(file_name: &str) -> Self {
        let raw: RawSimulationConfig = serde_yaml::from_str(
            &std::fs::read_to_string(file_name).unwrap_or_else(|_| panic!("Can't read file {}", file_name)),
        ).unwrap_or_else(|_| panic!("Can't parse YAML from file {}", file_name));

        Self {
            message_delay: raw.message_delay.unwrap_or(0.2),
            control_plane_message_delay: raw.control_plane_message_delay.unwrap_or(0.0),
            pod_start_duration: raw.pod_start_duration.unwrap_or(5.0),
            pod_stop_duration: raw.pod_stop_duration.unwrap_or(5.0),
            node_stop_duration: raw.node_stop_duration.unwrap_or(30.0),
            pod_initial_backoff_duration: raw.pod_initial_backoff_duration.unwrap_or(1.0),
            pod_max_backoff_duration: raw.pod_max_backoff_duration.unwrap_or(10.0),
            cluster_autoscaler_scan_interval: raw.cluster_autoscaler_scan_interval.unwrap_or(10.0),
            default_node: raw.default_node.unwrap_or(NodeConfig::new(8., 64., 1)),
            default_node_allocation_time: raw.default_node_allocation_time.unwrap_or(120.0),
            cloud_nodes_count: raw.cloud_nodes_count.unwrap_or(100),
            metrics_server_interval: raw.metrics_server_interval.unwrap_or(30.0),
            vpa_interval: raw.vpa_interval.unwrap_or(30.0),
            hpa_interval: raw.hpa_interval.unwrap_or(30.0),
            memory_pressure_threshold: raw.memory_pressure_threshold.unwrap_or(0.95),
            update_pods_resources_period: raw.update_pods_resources_period.unwrap_or(10.0),
            nodes: raw.nodes.unwrap_or_default(),
            trace: raw.trace,
        }
    }
}