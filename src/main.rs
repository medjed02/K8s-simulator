use std::time::Instant;
use dslab_core::Simulation;
use K8s_simulator::default_cluster_autoscaler_algorithms::default_simple_algorithm::SimpleClusterAutoscalerAlgorithm;
use K8s_simulator::default_horizontal_autoscaler_algorithms::default_horizontal_algorithm::ControlledResources::MemoryOnly;
use K8s_simulator::default_horizontal_autoscaler_algorithms::default_horizontal_algorithm::ResourcesHorizontalAutoscalerAlgorithm;
use K8s_simulator::default_scheduler_algorithms::lrp_algorithm::LRPAlgorithm;
use K8s_simulator::default_scheduler_algorithms::mrp_algorithm::MRPAlgorithm;
use K8s_simulator::default_vertical_autoscaler_algorithms::default_auto_algorithm::AutoVerticalAutoscalerAlgorithm;
use K8s_simulator::default_vertical_autoscaler_algorithms::default_auto_algorithm::ControlledValuesMode::{RequestsAndLimits, RequestsOnly};
use K8s_simulator::logger::StdoutLogger;
use K8s_simulator::simulation::K8sSimulation;
use K8s_simulator::simulation_config::SimulationConfig;
use K8s_simulator::simulation_metrics::{EmptyMetricsLogger, FileMetricsLogger};

fn main() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file("/home/medjed02/K8s-simulator/test-configs/alibaba_config.yaml");

    let mut k8s_sim = K8sSimulation::new(sim,
                                         Box::new(FileMetricsLogger::new(60.)),
                                         Box::new(StdoutLogger::new()),
                                         sim_config,
                                         Box::new(MRPAlgorithm::new()),
                                         None,
                                         None,
                                         Some(Box::new(ResourcesHorizontalAutoscalerAlgorithm::new(
                                             MemoryOnly {
                                                 memory_utilization: Some(0.2),
                                             }, 300.,  300.,1, 10))));

    k8s_sim.step_for_duration(1. * 86400.);

    k8s_sim.finish_simulation("/home/medjed02/K8s-simulator/results_with_hpa.json").unwrap();
}