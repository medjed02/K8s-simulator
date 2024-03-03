use dslab_core::Simulation;
use K8s_simulator::default_cluster_autoscaler_algorithms::default_simple_algorithm::SimpleClusterAutoscalerAlgorithm;
use K8s_simulator::default_scheduler_algorithms::mrp_algorithm::MRPAlgorithm;
use K8s_simulator::default_scheduler_algorithms::lrp_algorithm::LRPAlgorithm;
use K8s_simulator::node::NodeState;
use K8s_simulator::simulation::K8sSimulation;
use K8s_simulator::simulation_config::SimulationConfig;

fn name_wrapper(file_name: &str) -> String {
    format!("test-configs/{}", file_name)
}

fn get_default_simulation_with_mrp() -> K8sSimulation {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    K8sSimulation::new(sim, sim_config, Box::new(MRPAlgorithm::new()),
                       Box::new(SimpleClusterAutoscalerAlgorithm::new(
                           600.0,
                           10,
                           300.0
                       )),
                       false, 0)
}

#[test]
fn test_base_simulation_with_mrp() {
    let mut k8s_sim = get_default_simulation_with_mrp();
    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100, 1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.submit_pod(16.0, 10., 16.0, 10., 100, 1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 20.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 20.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);
}

#[test]
fn test_base_simulation_with_lrp() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(LRPAlgorithm::new()),
                       Box::new(SimpleClusterAutoscalerAlgorithm::new(
                           600.0,
                           10,
                           300.0
                       )), false, 0);

    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100, 1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100, 1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 10.0);
}

#[test]
fn test_pod_removing() {
    let mut k8s_sim = get_default_simulation_with_mrp();
    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    let pod_id_1 = k8s_sim.submit_pod(5.0, 5.0, 5.0, 5.0, 100, 1.);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.remove_pod(pod_id_1);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);
}

#[test]
fn test_node_crashing() {
    let mut k8s_sim = get_default_simulation_with_mrp();

    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(5.0, 5.0, 5.0, 5.0, 100, 1.);
    k8s_sim.step_for_duration(30.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.crash_node(node_id_1, 1.0);
    k8s_sim.step_for_duration(30.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().state, NodeState::Failed);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().state, NodeState::Working);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 5.0);

    k8s_sim.recover_node(node_id_1, 1.0);
    k8s_sim.step_for_duration(30.0);
    k8s_sim.crash_node(node_id_2, 1.0);
    k8s_sim.step_for_duration(30.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().state, NodeState::Working);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().state, NodeState::Failed);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);
}

#[test]
fn base_test_unschedulable_pod() {
    let mut k8s_sim = get_default_simulation_with_mrp();

    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(30.0, 30.0, 30.0, 30.0, 100, 1.);
    k8s_sim.step_for_duration(0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    let node_id_3 = k8s_sim.add_node(100, 100);
    k8s_sim.step_for_duration(100.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_3).borrow().cpu_load, 30.0);
    assert_eq!(k8s_sim.node(node_id_3).borrow().memory_load, 30.0);
}

#[test]
fn test_cluster_scale_up() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(MRPAlgorithm::new()),
                                         Box::new(SimpleClusterAutoscalerAlgorithm::new(
                                             600.0,
                                             10,
                                             300.0
                                         )), true, 10);

    k8s_sim.submit_pod(2.0, 6.0, 2.0, 6.0, 100, 1.);
    k8s_sim.step_for_duration(300.0);
    assert_ne!(k8s_sim.working_nodes().len(), 0);
    assert_ne!(k8s_sim.cpu_load_rate(), 0.0);
    assert_ne!(k8s_sim.memory_load_rate(), 0.0);
}

#[test]
fn test_cluster_scale_down() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(MRPAlgorithm::new()),
                                         Box::new(SimpleClusterAutoscalerAlgorithm::new(
                                             600.0,
                                             10,
                                             300.0
                                         )), true, 10);
    let node_id_1 = k8s_sim.add_node(20, 20);
    assert_ne!(k8s_sim.working_nodes().len(), 0);
    
    k8s_sim.step_for_duration(700.0);
    assert_eq!(k8s_sim.working_nodes().len(), 0);
    assert_eq!(k8s_sim.failed_nodes().len(), 0);
}