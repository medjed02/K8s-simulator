use dslab_core::Simulation;
use K8s_simulator::simulation::K8sSimulation;
use K8s_simulator::simulation_config::SimulationConfig;

#[test]
fn test_base_simulation_with_mrp() {
    let sim = Simulation::new(42);
    let sim_config = SimulationConfig::new(2., 0., 0., 0.);
    let mut k8s_sim = K8sSimulation::new(sim, sim_config);
    k8s_sim.add_node(4, 18);
    k8s_sim.add_node(1, 2);
    k8s_sim.submit_pod(1., 1., 1., 1., 100, 1.);
    println!("{}", k8s_sim.average_memory_load());
    k8s_sim.step_until_no_events();
    println!("{} {}", k8s_sim.memory_load_rate(), k8s_sim.current_time());
}