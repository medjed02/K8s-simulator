# K8s-simulator

Here is Kubernetes simulator.
Nowadays, the work with simulator looks like this:
1) Setting the initial parameters of the [simulation](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/simulation.rs#L48) (both using the config and by transferring the algorithms of the components through the parameters of the simulation object).
2) Setting general simulation events (sending pods, deployments) is possible both through [K8sSimulation functions](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/simulation.rs#L31) and through [reading the trace](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/dataset_reader.rs#L46).
3) Start the simulation using the simulation execution control functions (mainly, [step_for_duration](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/simulation.rs#L313C12-L313C29).
4) The end of the simulation, saving the metrics collected during the simulation ([finish_simulation](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/simulation.rs#L298C12-L298C29)).

You can implement k8s components according to the traits ([SchedulerAlgorithm](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/scheduler_algorithm.rs#L7C11-L7C30), [ClusterAutoscalerAlgorithm](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/cluster_autoscaler_algorithm.rs#L8C11-L8C37), [VerticalAutoscalerAlgorithm](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/vertical_autoscaler_algorithm.rs#L16C11-L16C39), [HorizontalAutoscalerAlgorithm](https://github.com/medjed02/K8s-simulator/blob/1828593281f23335e2a4f33f537c8c4efd023ff0/src/horizontal_autoscaler_algorithm.rs#L4C11-L4C41)). After implementation, you just need these to initial parameters of K8sSimulation object. There are some default algorithms (in the [src](https://github.com/medjed02/K8s-simulator/tree/base-objects/src) directory) of these components, you can use them.

## Example of usage
```
fn main() {
    // create base object of simulation (from dslab-core)
    let sim = Simulation::new(42);

    // load simulation config
    let sim_config = SimulationConfig::from_file("./config.yaml");

    // create K8sSimulation, send to parameters implementations of k8s components
    let mut k8s_sim = K8sSimulation::new(sim,
                                         Box::new(FileMetricsLogger::new(60.)),
                                         Box::new(StdoutLogger::new()),
                                         sim_config,
                                         Box::new(MRPAlgorithm::new()),
                                         Some(Box::new(SimpleClusterAutoscalerAlgorithm::new(300.0, 10, 300.0))),
                                         Some(Box::new(AutoVerticalAutoscalerAlgorithm::new(RequestsOnly))),
                                         Some(Box::new(ResourcesHorizontalAutoscalerAlgorithm::new(
                                             MemoryOnly {
                                                 memory_utilization: Some(0.5),
                                             }, 300.,  300., 1, 7))));
    // add node
    k8s_sim.add_node(16., 64.);

    // add pod
    k8s_sim.submit_pod(4.0, 10., 4.0, 10., 100,
                       Box::new(ConstantLoadModel::new(4.0)),
                       Box::new(ConstantLoadModel::new(10.0)), 1.);

    // add deployment
    k8s_sim.submit_deployment(5., 10., 5., 10., 100,
                              Box::new(ConstantLoadModel::new(5.)),
                              Box::new(ConstantLoadModel::new(10.0)),
                              1, 1.);

    // control simulation time
    k8s_sim.step_for_duration(1000.);

    // finish simulation, save metrics results to the file
    k8s_sim.finish_simulation("./results.json").unwrap();
}
```
