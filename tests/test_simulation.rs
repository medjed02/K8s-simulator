use dslab_core::Simulation;
use K8s_simulator::default_scheduler_algorithms::mrp_algorithm::MRPAlgorithm;
use K8s_simulator::default_scheduler_algorithms::lrp_algorithm::LRPAlgorithm;
use K8s_simulator::node::NodeState;
use K8s_simulator::simulation::K8sSimulation;
use K8s_simulator::simulation_config::SimulationConfig;

fn name_wrapper(file_name: &str) -> String {
    format!("test-configs/{}", file_name)
}

#[test]
fn test_base_simulation_with_mrp() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(MRPAlgorithm::new()));
    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100, 1.);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.submit_pod(16.0, 10., 16.0, 10., 100, 1.);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 20.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 20.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);
}

#[test]
fn test_base_simulation_with_lrp() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(LRPAlgorithm::new()));
    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100, 1.);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100, 1.);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 10.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 4.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 10.0);
}

#[test]
fn test_pod_removing() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(MRPAlgorithm::new()));
    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    let pod_id_1 = k8s_sim.submit_pod(5.0, 5.0, 5.0, 5.0, 100, 1.);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.remove_pod(pod_id_1);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);
}

#[test]
fn test_node_crashing() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(MRPAlgorithm::new()));
    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(5.0, 5.0, 5.0, 5.0, 100, 1.);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);

    k8s_sim.remove_node(node_id_1, 1.0);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().state, NodeState::Failed);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().state, NodeState::Working);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 5.0);

    k8s_sim.recover_node(node_id_1, 1.0);
    k8s_sim.step_until_no_events();
    k8s_sim.remove_node(node_id_2, 1.0);
    k8s_sim.step_until_no_events();
    assert_eq!(k8s_sim.node(node_id_1).borrow().state, NodeState::Working);
    assert_eq!(k8s_sim.node(node_id_1).borrow().cpu_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_1).borrow().memory_load, 5.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().state, NodeState::Failed);
    assert_eq!(k8s_sim.node(node_id_2).borrow().cpu_load, 0.0);
    assert_eq!(k8s_sim.node(node_id_2).borrow().memory_load, 0.0);
}

#[test]
fn base_test_unschedulable_pod() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::from_file(&name_wrapper("config.yaml"));
    let mut k8s_sim = K8sSimulation::new(sim, sim_config, Box::new(MRPAlgorithm::new()));
    let node_id_1 = k8s_sim.add_node(20, 20);
    let node_id_2 = k8s_sim.add_node(20, 20);

    k8s_sim.submit_pod(30.0, 30.0, 30.0, 30.0, 100, 1.);
    k8s_sim.step_for_duration(30.0);
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