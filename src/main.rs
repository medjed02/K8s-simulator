use std::time::Instant;
use dslab_core::Simulation;
use K8s_simulator::default_cluster_autoscaler_algorithms::default_simple_algorithm::SimpleClusterAutoscalerAlgorithm;
use K8s_simulator::default_horizontal_autoscaler_algorithms::default_horizontal_algorithm::ControlledResources::MemoryOnly;
use K8s_simulator::default_horizontal_autoscaler_algorithms::default_horizontal_algorithm::ResourcesHorizontalAutoscalerAlgorithm;
use K8s_simulator::default_scheduler_algorithms::lrp_algorithm::LRPAlgorithm;
use K8s_simulator::default_scheduler_algorithms::mrp_algorithm::MRPAlgorithm;
use K8s_simulator::default_vertical_autoscaler_algorithms::default_auto_algorithm::AutoVerticalAutoscalerAlgorithm;
use K8s_simulator::default_vertical_autoscaler_algorithms::default_auto_algorithm::ControlledValuesMode::{RequestsAndLimits, RequestsOnly};
use K8s_simulator::load_model::{ConstantLoadModel, ResourceSnapshot, TraceLoadModel};
use K8s_simulator::logger::StdoutLogger;
use K8s_simulator::simulation::K8sSimulation;
use K8s_simulator::simulation_config::SimulationConfig;
use K8s_simulator::simulation_metrics::{EmptyMetricsLogger, FileMetricsLogger};

fn main() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file("/home/medjed02/K8s-simulator/test-configs/config.yaml");

    let mut k8s_sim = K8sSimulation::new(sim,
                                         Box::new(FileMetricsLogger::new(10.)),
                                         Box::new(StdoutLogger::new()),
                                         sim_config,
                                         Box::new(MRPAlgorithm::new()),
                                         None,
                                         None,
                                         Some(Box::new(ResourcesHorizontalAutoscalerAlgorithm::new(
                                             MemoryOnly {
                                                 memory_utilization: Some(0.2),
                                             }, 10.,  30.,1, 10))));
    k8s_sim.add_node(100.0, 100.0);

    let history = vec![ResourceSnapshot{timestamp: 0., resource: 0.4},
                       ResourceSnapshot{timestamp: 10., resource: 0.4},
                       ResourceSnapshot{timestamp: 20., resource: 0.5},
                       ResourceSnapshot{timestamp: 30., resource: 0.6},
                       ResourceSnapshot{timestamp: 40., resource: 0.7},
                       ResourceSnapshot{timestamp: 50., resource: 0.8},
                       ResourceSnapshot{timestamp: 60., resource: 0.9},
                       ResourceSnapshot{timestamp: 70., resource: 1.0},
                       ResourceSnapshot{timestamp: 80., resource: 1.1},
                       ResourceSnapshot{timestamp: 90., resource: 1.2},
                       ResourceSnapshot{timestamp: 100., resource: 1.3},
                       ResourceSnapshot{timestamp: 110., resource: 1.4},
                       ResourceSnapshot{timestamp: 120., resource: 1.5},
                       ResourceSnapshot{timestamp: 130., resource: 1.6},
                       ResourceSnapshot{timestamp: 140., resource: 1.7},
                       ResourceSnapshot{timestamp: 150., resource: 1.8},
                       ResourceSnapshot{timestamp: 160., resource: 1.9},
                       ResourceSnapshot{timestamp: 170., resource: 2.0},
                       ResourceSnapshot{timestamp: 200., resource: 1.9},
                       ResourceSnapshot{timestamp: 210., resource: 1.8},
                       ResourceSnapshot{timestamp: 220., resource: 1.7},
                       ResourceSnapshot{timestamp: 230., resource: 1.6},
                       ResourceSnapshot{timestamp: 240., resource: 1.5},
                       ResourceSnapshot{timestamp: 250., resource: 1.4},
                       ResourceSnapshot{timestamp: 260., resource: 1.3},
                       ResourceSnapshot{timestamp: 270., resource: 1.2},
                       ResourceSnapshot{timestamp: 280., resource: 1.1},
                       ResourceSnapshot{timestamp: 290., resource: 1.0}];
    let memory_load_model = TraceLoadModel::new(history);
    k8s_sim.submit_deployment(0.5, 2.0, 0.5, 2.0, 1,
                              Box::new(ConstantLoadModel::new(0.5)), Box::new(memory_load_model), 1, 0.0);

    k8s_sim.step_for_duration(400.);

    k8s_sim.finish_simulation("/home/medjed02/K8s-simulator/test_results_with_hpa.json").unwrap();
}