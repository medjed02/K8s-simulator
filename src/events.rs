//! Standard simulation events.

// POD ASSIGNING EVENTS //
pub mod assigning {
    use serde::Serialize;
    use crate::pod::Pod;

    #[derive(Clone, Serialize)]
    pub struct PodAssigningRequest {
        pub pod: Pod,
    }

    #[derive(Clone, Serialize)]
    pub struct PodAssigningSucceeded {
        pub pod: Pod,
        pub node_id: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct PodAssigningFailed {
        pub pod: Pod,
        pub scheduling_cycle: i64,
    }

    #[derive(Clone, Serialize)]
    pub struct PodPlacementRequest {
        pub pod: Pod,
        pub node_id: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct PodPlacementSucceeded {
        pub pod_id: u64,
        pub node_id: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct PodPlacementFailed {
        pub pod_id: u64,
        pub node_id: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct PodMigrationRequest {
        pub pod_id: u64,
        pub source_node_id: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct PodMigrationSucceeded {
        pub pod_id: u64,
        pub source_node_id: u32,
        pub distance_node_id: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct PodMigrationFailed {
        pub pod_id: u64,
        pub source_node_id: u32,
    }
}

// NODE CHANGING STATUS EVENTS //
pub mod node {
    use serde::Serialize;
    use crate::node::{NodeState};

    #[derive(Clone, Serialize)]
    pub struct AllocateNewDefaultNodes {
        pub cnt_nodes: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct RemoveNode {
        pub node_id: u32,
    }

    #[derive(Clone, Serialize)]
    pub struct NodeStatusChanged {
        pub node_id: u32,
        pub new_status: NodeState,
    }
}

// SCHEDULER'S WORK EVENTS //
pub mod scheduler {
    use serde::Serialize;
    use crate::pod::Pod;

    #[derive(Clone, Serialize)]
    pub struct SchedulingCycle {
    }

    #[derive(Clone, Serialize)]
    pub struct PodBackoffRetry {
        pub pod: Pod,
    }

    #[derive(Clone, Serialize)]
    pub struct FlushUnschedulableQueue {
    }

    #[derive(Clone, Serialize)]
    pub struct MoveRequest {
    }
}

// API SERVER INTERACTION EVENTS //
pub mod api_server {
    use serde::Serialize;

    #[derive(Clone, Serialize)]
    pub struct PodRemoveRequest {
        pub pod_id: u64,
    }
}

pub mod autoscaler {
    use serde::Serialize;

    #[derive(Clone, Serialize)]
    pub struct ClusterAutoscalerScan {
    }
}