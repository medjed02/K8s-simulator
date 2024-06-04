use dslab_core::Simulation;
use K8s_simulator::default_cluster_autoscaler_algorithms::default_simple_algorithm::SimpleClusterAutoscalerAlgorithm;
use K8s_simulator::default_horizontal_autoscaler_algorithms::default_horizontal_algorithm::ControlledResources::CPUOnly;
use K8s_simulator::default_horizontal_autoscaler_algorithms::default_horizontal_algorithm::ResourcesHorizontalAutoscalerAlgorithm;
use K8s_simulator::default_scheduler_algorithms::mrp_algorithm::MRPAlgorithm;
use K8s_simulator::default_scheduler_algorithms::lrp_algorithm::LRPAlgorithm;
use K8s_simulator::default_vertical_autoscaler_algorithms::default_auto_algorithm::AutoVerticalAutoscalerAlgorithm;
use K8s_simulator::default_vertical_autoscaler_algorithms::default_auto_algorithm::ControlledValuesMode::RequestsAndLimits;
use K8s_simulator::load_model::{ConstantLoadModel, DecreaseLoadModel, IncreaseLoadModel};
use K8s_simulator::logger::StdoutLogger;
use K8s_simulator::node::NodeState;
use K8s_simulator::simulation::K8sSimulation;
use K8s_simulator::simulation_config::SimulationConfig;
use K8s_simulator::simulation_metrics::{EmptyMetricsLogger, StdoutMetricsLogger};

fn name_wrapper(file_name: &str) -> String {
    format!("test-configs/{}", file_name)
}

fn get_default_simulation_with_mrp() -> K8sSimulation {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    K8sSimulation::new(sim, Box::new(EmptyMetricsLogger {}), Box::new(StdoutLogger::new()),
                       sim_config, Box::new(MRPAlgorithm::new()), None, None, None)
}

#[test]
fn test_base_simulation_with_mrp() {
    let mut k8s_sim = get_default_simulation_with_mrp();
    let node_id_1 = k8s_sim.add_node(20., 20.);
    let node_id_2 = k8s_sim.add_node(20., 20.);

    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100,
                       Box::new(ConstantLoadModel::new(4.0)),
                       Box::new(ConstantLoadModel::new(10.0))
                       , 1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);

    k8s_sim.submit_pod(4., 5., 4.0, 5., 100,
                       Box::new(ConstantLoadModel::new(4.0)),
                       Box::new(ConstantLoadModel::new(5.0)),
                       1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 8.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 15.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);
}

#[test]
fn test_base_simulation_with_lrp() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, Box::new(EmptyMetricsLogger {}), Box::new(StdoutLogger::new()),
                                         sim_config, Box::new(LRPAlgorithm::new()), None, None, None);

    let node_id_1 = k8s_sim.add_node(20., 20.);
    let node_id_2 = k8s_sim.add_node(20., 20.);

    k8s_sim.submit_pod(4.0, 5., 4.0, 5., 100,
                       Box::new(ConstantLoadModel::new(4.0)),
                       Box::new(ConstantLoadModel::new(5.0)),
                       1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);

    k8s_sim.submit_pod(4.0, 5., 4.0, 5., 100,
                       Box::new(ConstantLoadModel::new(4.0)),
                       Box::new(ConstantLoadModel::new(5.0)),
                       1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 4.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 5.0);
}

#[test]
fn test_pod_removing() {
    let mut k8s_sim = get_default_simulation_with_mrp();
    let node_id_1 = k8s_sim.add_node(20., 20.);
    let node_id_2 = k8s_sim.add_node(20., 20.);

    let pod_id_1 = k8s_sim.submit_pod(5.0, 5.0, 5.0, 5.0, 100,
                                      Box::new(ConstantLoadModel::new(5.0)),
                                      Box::new(ConstantLoadModel::new(5.0)),
                                      1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);

    k8s_sim.remove_pod(pod_id_1);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);
}

#[test]
fn test_node_crashing() {
    let mut k8s_sim = get_default_simulation_with_mrp();

    let node_id_1 = k8s_sim.add_node(20., 20.);
    let node_id_2 = k8s_sim.add_node(20., 20.);

    k8s_sim.submit_pod(5.0, 5.0, 5.0, 5.0, 100,
                       Box::new(ConstantLoadModel::new(5.0)),
                       Box::new(ConstantLoadModel::new(5.0)),
                       1.);
    k8s_sim.step_for_duration(30.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);

    k8s_sim.crash_node(node_id_1, 1.0);
    k8s_sim.step_for_duration(30.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().state, NodeState::Failed);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().state, NodeState::Working);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 5.0);

    k8s_sim.recover_node(node_id_1, 1.0);
    k8s_sim.step_for_duration(30.0);
    k8s_sim.crash_node(node_id_2, 1.0);
    k8s_sim.step_for_duration(30.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().state, NodeState::Working);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().state, NodeState::Failed);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);
}

#[test]
fn base_test_unschedulable_pod() {
    let mut k8s_sim = get_default_simulation_with_mrp();

    let node_id_1 = k8s_sim.add_node(20., 20.);
    let node_id_2 = k8s_sim.add_node(20., 20.);

    k8s_sim.submit_pod(30.0, 30.0, 30.0, 30.0, 100,
                       Box::new(ConstantLoadModel::new(30.0)),
                       Box::new(ConstantLoadModel::new(30.0)),
                       1.);
    k8s_sim.step_for_duration(0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);

    let node_id_3 = k8s_sim.add_node(100., 100.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id_3).borrow().cpu_allocated, 30.0);
    assert_eq!(k8s_sim.node(node_id_3).borrow().memory_allocated, 30.0);
}

#[test]
fn test_cluster_scale_up() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, Box::new(EmptyMetricsLogger {}), Box::new(StdoutLogger::new()),
                                         sim_config, Box::new(MRPAlgorithm::new()),
                                         Some(Box::new(SimpleClusterAutoscalerAlgorithm::new(
                                             600.0,
                                             10,
                                             300.0
                                         ))), None, None);

    k8s_sim.submit_pod(2.0, 6.0, 2.0, 6.0, 100,
                       Box::new(ConstantLoadModel::new(2.0)),
                       Box::new(ConstantLoadModel::new(6.0)),
                       1.);
    k8s_sim.step_for_duration(1000.0);
    assert_ne!(k8s_sim.working_nodes().len(), 0);
    assert_ne!(k8s_sim.cpu_allocated_load_rate(), 0.0);
    assert_ne!(k8s_sim.memory_allocated_load_rate(), 0.0);
}

#[test]
fn test_cluster_scale_down() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, Box::new(EmptyMetricsLogger {}), Box::new(StdoutLogger::new()),
                                         sim_config, Box::new(MRPAlgorithm::new()),
                                         Some(Box::new(SimpleClusterAutoscalerAlgorithm::new(
                                             600.0,
                                             10,
                                             300.0
                                         ))), None, None);
    let node_id_1 = k8s_sim.add_node(20., 20.);
    assert_ne!(k8s_sim.working_nodes().len(), 0);

    k8s_sim.step_for_duration(700.0);
    assert_eq!(k8s_sim.working_nodes().len(), 0);
    assert_eq!(k8s_sim.failed_nodes().len(), 0);
}

#[test]
fn test_pod_load_model() {
    let mut k8s_sim = get_default_simulation_with_mrp();
    let node_id = k8s_sim.add_node(20., 20.);

    let pod_id = k8s_sim.submit_pod(4.0, 10., 8.0, 20., 100,
                       Box::new(IncreaseLoadModel::new(100.0, 4.0, 10.0)),
                       Box::new(IncreaseLoadModel::new(100.0, 10.0, 30.0)),
                                    1.);
    k8s_sim.step_for_duration(20.0);
    assert!(k8s_sim.node(node_id).borrow().cpu_allocated < 8.0);
    assert!(k8s_sim.node(node_id).borrow().memory_allocated < 20.0);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id).borrow().cpu_allocated, 8.0);
    assert_eq!(k8s_sim.node(node_id).borrow().memory_allocated, 20.0);
    k8s_sim.remove_pod(pod_id);
    k8s_sim.step_for_duration(100.0);

    let pod_id = k8s_sim.submit_pod(4.0, 10., 8.0, 20., 100,
                                    Box::new(DecreaseLoadModel::new(100.0, 8.0, 2.0)),
                                    Box::new(DecreaseLoadModel::new(100.0, 20.0, 0.0)),
                                    1.);
    k8s_sim.step_for_duration(20.0);
    assert!(k8s_sim.node(node_id).borrow().cpu_allocated > 4.0);
    assert!(k8s_sim.node(node_id).borrow().memory_allocated > 10.0);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id).borrow().cpu_allocated, 4.0);
    assert_eq!(k8s_sim.node(node_id).borrow().memory_allocated, 10.0);
    k8s_sim.remove_pod(pod_id);
    k8s_sim.step_for_duration(100.0);

    assert_eq!(k8s_sim.node(node_id).borrow().cpu_allocated, 0.0);
    assert_eq!(k8s_sim.node(node_id).borrow().memory_allocated, 0.0);
}

#[test]
fn test_vertical_autoscaler() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, Box::new(EmptyMetricsLogger {}), Box::new(StdoutLogger::new()),
                                         sim_config, Box::new(MRPAlgorithm::new()),
                                         None,
                                         Some(Box::new(AutoVerticalAutoscalerAlgorithm::new(RequestsAndLimits))),
                                         None);
    let node_id = k8s_sim.add_node(20., 20.);
    let pod_id = k8s_sim.submit_pod(10.0, 10.0, 10.0, 10.0, 100,
                       Box::new(ConstantLoadModel::new(1.0)),
                       Box::new(ConstantLoadModel::new(1.0)),
                                    1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id).borrow().cpu_allocated, 10.0);
    assert_eq!(k8s_sim.node(node_id).borrow().memory_allocated, 10.0);

    k8s_sim.step_for_duration(40000.0);
    assert!(k8s_sim.node(node_id).borrow().cpu_allocated < 2.0);
    assert!(k8s_sim.node(node_id).borrow().memory_allocated < 2.0);

    k8s_sim.remove_pod(pod_id);
    let pod_id = k8s_sim.submit_pod(10.0, 10.0, 15.0, 15.0, 100,
                                    Box::new(ConstantLoadModel::new(12.0)),
                                    Box::new(ConstantLoadModel::new(12.0)),
                                    1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id).borrow().cpu_allocated, 12.0);
    assert_eq!(k8s_sim.node(node_id).borrow().memory_allocated, 12.0);

    k8s_sim.step_for_duration(40000.0);
    assert!(k8s_sim.node(node_id).borrow().pods.get(&pod_id).unwrap().requested_cpu > 10.0);
    assert!(k8s_sim.node(node_id).borrow().pods.get(&pod_id).unwrap().requested_memory > 10.0);
}

#[test]
fn test_create_deployment() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, Box::new(EmptyMetricsLogger {}), Box::new(StdoutLogger::new()),
                                         sim_config, Box::new(LRPAlgorithm::new()), None, None, None);

    let node_id_1 = k8s_sim.add_node(5., 20.);
    let node_id_2 = k8s_sim.add_node(5., 20.);

    let deployment_id = k8s_sim.submit_deployment(5., 10., 5., 10., 100,
                                                  Box::new(ConstantLoadModel::new(5.0)),
                                                  Box::new(ConstantLoadModel::new(10.0)),
                                                  2, 1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_allocated, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_allocated, 10.0);
}

#[test]
fn test_horizontal_autoscaler() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let horizontal_autoscaler =
        Box::new(
            ResourcesHorizontalAutoscalerAlgorithm::new(
                CPUOnly { cpu_utilization: Some(0.5) }, 300.0, 300.0,1, 10
            )
        );
    let mut k8s_sim = K8sSimulation::new(sim, Box::new(EmptyMetricsLogger {}), Box::new(StdoutLogger::new()),
                                         sim_config, Box::new(LRPAlgorithm::new()),
                                         None, None, Some(horizontal_autoscaler));
    let node_id_1 = k8s_sim.add_node(5., 20.);
    let node_id_2 = k8s_sim.add_node(5., 20.);

    k8s_sim.submit_deployment(5., 10., 5., 10., 100,
                              Box::new(ConstantLoadModel::new(5.)),
                              Box::new(ConstantLoadModel::new(10.0)),
                              1, 1.);
    k8s_sim.step_for_duration(1000.0);

    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_used, 2.5);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_allocated, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_used, 2.5);

}