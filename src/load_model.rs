//! Resource load models.

use dyn_clone::{clone_trait_object, DynClone};
use erased_serde::serialize_trait_object;
use serde::Serialize;


/// A resource load model is a function, which defines load of resource X at the moment.
/// time - current simulation time, time_from_start - time from previous initialization
/// which allows to model load peak at the beginning of Pod lifecycle.
/// This time is dropped to zero when Pod is migrated.
pub trait LoadModel: DynClone + erased_serde::Serialize {
    fn get_resource(&mut self, time: f64, time_from_start: f64, cnt_replicas: u64) -> f64;
}

clone_trait_object!(LoadModel);
serialize_trait_object!(LoadModel);

#[derive(Clone, Serialize)]
pub struct ConstantLoadModel {
    resource: f64,
}

impl ConstantLoadModel {
    pub fn new(resource: f64) -> Self {
        Self { resource }
    }
}

impl LoadModel for ConstantLoadModel {
    fn get_resource(&mut self, _time: f64, _time_from_start: f64, cnt_replicas: u64) -> f64 {
        self.resource / cnt_replicas as f64
    }
}

#[derive(Clone, Serialize)]
pub struct DecreaseLoadModel {
    decrease_time: f64,
    start_resource: f64,
    end_resource: f64,
}

impl DecreaseLoadModel {
    pub fn new(decrease_time: f64, start_resource: f64, end_resource: f64) -> Self {
        assert!(0.0 <= end_resource && end_resource <= start_resource);
        assert!(decrease_time > 0.0);
        Self { decrease_time, start_resource, end_resource }
    }
}

impl LoadModel for DecreaseLoadModel {
    fn get_resource(&mut self, _time: f64, _time_from_start: f64, cnt_replicas: u64) -> f64 {
        (self.start_resource - (_time_from_start / self.decrease_time).min(1.0) *
            (self.start_resource - self.end_resource)) / cnt_replicas as f64
    }
}

#[derive(Clone, Serialize)]
pub struct IncreaseLoadModel {
    increase_time: f64,
    start_resource: f64,
    end_resource: f64,
}

impl IncreaseLoadModel {
    pub fn new(increase_time: f64, start_resource: f64, end_resource: f64) -> Self {
        assert!(0.0 <= start_resource && start_resource <= end_resource);
        assert!(increase_time > 0.0);
        Self { increase_time, start_resource, end_resource }
    }
}

impl LoadModel for IncreaseLoadModel {
    fn get_resource(&mut self, _time: f64, _time_from_start: f64, cnt_replicas: u64) -> f64 {
        (self.start_resource + (_time_from_start / self.increase_time).min(1.0) *
            (self.end_resource - self.start_resource)) / cnt_replicas as f64
    }
}

#[derive(Clone, Serialize)]
pub struct ResourceSnapshot {
    pub timestamp: f64,
    pub resource: f64,
}

#[derive(Clone, Default, Serialize)]
pub struct TraceLoadModel {
    resource_history: Vec<ResourceSnapshot>,
    now_ptr: usize,
}

impl TraceLoadModel {
    pub fn new(resource_history: Vec<ResourceSnapshot>) -> Self {
        Self { resource_history, now_ptr: 0 }
    }

    pub fn get_now_resource_snapshot(&mut self, now_ptr: usize, timestamp: f64) -> usize {
        let mut ptr = now_ptr;
        while ptr + 1 < self.resource_history.len() {
            if self.resource_history[ptr + 1].timestamp > timestamp {
                break
            }
            ptr += 1;
        }
        ptr
    }
}

impl LoadModel for TraceLoadModel {
    fn get_resource(&mut self, time: f64, time_from_start: f64, cnt_replicas: u64) -> f64 {
        let timestamp = time - time_from_start;
        if self.resource_history[self.now_ptr].timestamp > timestamp {
            self.now_ptr = 0;
        }
        self.now_ptr = self.get_now_resource_snapshot(self.now_ptr, timestamp);
        self.resource_history[self.now_ptr].resource / cnt_replicas as f64
    }
}
