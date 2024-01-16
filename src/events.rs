//! Standard simulation events.

// POD ASSIGNING EVENTS //
pub mod assigning {
    use crate::pod::Pod;

    #[derive(Clone)]
    pub struct PodAssigningRequest {
        pub pod: Pod,
    }

    #[derive(Clone)]
    pub struct PodAssigningSucceeded {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodAssigningFailed {
        pub pod_id: u64,
    }

    #[derive(Clone)]
    pub struct PodPlacementRequest {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodPlacementSucceeded {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodPlacementFailed {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodMigrationRequest {
        pub pod_id: u64,
        pub source_node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodMigrationSucceeded {
        pub pod_id: u64,
        pub source_node_id: u64,
        pub distance_node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodMigrationFailed {
        pub pod_id: u64,
        pub source_node_id: u64,
    }
}

// NODE CHANGING STATUS EVENTS //
pub mod node {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::node::{Node, NodeState};

    #[derive(Clone)]
    pub struct NodeStatusChanged {
        pub node_id: u64,
        pub new_status: NodeState,
    }

    #[derive(Clone)]
    pub struct NewNodeAdded {
        pub node: Rc<RefCell<Node>>,
    }
}

// SCHEDULER'S WORK EVENTS //
pub mod scheduler {

}

// API SERVER INTERACTION EVENTS //
pub mod api_server {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::node::Node;
    use crate::pod::Pod;

    #[derive(Clone)]
    pub struct GetPodRequest {
    }

    #[derive(Clone)]
    pub struct GetPodResponse {
        pub pod: Option<Pod>,
    }

    #[derive(Clone)]
    pub struct GetNodesRequest {
    }

    #[derive(Clone)]
    pub struct GetNodesResponse {
        pub nodes: Rc<RefCell<HashMap<u64, Rc<RefCell<Node>>>>>,
    }
}