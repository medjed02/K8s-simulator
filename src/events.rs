//! Standard simulation events.

// POD ASSIGNING EVENTS //
pub mod assigning {
    #[derive(Clone)]
    pub struct PodAssigningRequest {
        pub pod_id: u64,
    }

    #[derive(Clone)]
    pub struct PodAssigningCommitRequest {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodAssigningCommitSucceeded {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodAssigningCommitFailed {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodAssigningFailed {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodAssigningSucceeded {
        pub pod_id: u64,
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct PodMigrationRequest {
        pub pod_id: u64,
        pub source_node_id: u64,
    }
}

// NODE CHANGING STATUS EVENTS //
pub mod node {
    #[derive(Clone)]
    pub struct NodeWorking {
        pub node_id: u64,
    }

    #[derive(Clone)]
    pub struct NodeFailed {
        pub node_id: u64,
    }
}

// SCHEDULER'S WORK EVENTS //
pub mod scheduler {

}

// API SERVER INTERACTION EVENTS //
pub mod api_server {
    use crate::node::Node;
    use crate::pod::Pod;

    #[derive(Clone)]
    pub struct GetPodRequest {
    }

    #[derive(Clone)]
    pub struct GetPodResponse {
        pub pod: Pod,
    }

    #[derive(Clone)]
    pub struct GetNodesRequest {
    }

    #[derive(Clone)]
    pub struct GetNodesResponse {
        // TODO: need a pointer, not vec
        pub nodes: Vec<Node>,
    }
}