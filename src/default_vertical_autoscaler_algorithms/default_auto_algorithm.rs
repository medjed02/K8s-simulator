use std::cell::RefCell;
use std::rc::Rc;
use serde::Serialize;
use crate::default_vertical_autoscaler_algorithms::default_auto_algorithm::ControlledValuesMode::RequestsAndLimits;
use crate::metrics_server::PodStatistic;
use crate::node::Node;
use crate::pod::Pod;
use crate::vertical_autoscaler_algorithm::{VerticalAutoscalerAlgorithm, VPARecommendation};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ControlledValuesMode {
    RequestsOnly,
    RequestsAndLimits,
}

pub struct AutoVerticalAutoscalerAlgorithm {
    controlled_values: ControlledValuesMode,
}

impl AutoVerticalAutoscalerAlgorithm {
    pub fn new(controlled_values: ControlledValuesMode) -> Self {
        Self {
            controlled_values
        }
    }
}

const SECONDS_PER_DAY: f64 = 60.0 * 60.0 * 24.0;
const DEFAULT_LOWER_BOUND_CPU_PERCENTILE: f64 = 0.5;
const DEFAULT_TARGET_CPU_PERCENTILE: f64 = 0.9;
const DEFAULT_UPPER_BOUND_CPU_PERCENTILE: f64 = 0.95;
const DEFAULT_LOWER_BOUND_MEMORY_PERCENTILE: f64 = 0.5;
const DEFAULT_TARGET_MEMORY_PERCENTILE: f64 = 0.9;
const DEFAULT_UPPER_BOUND_MEMORY_PERCENTILE: f64 = 0.95;

impl VerticalAutoscalerAlgorithm for AutoVerticalAutoscalerAlgorithm {
    fn get_recommendation(&mut self, pod: &Pod, pod_statistic: PodStatistic) -> Option<VPARecommendation> {
        let period_in_days = pod_statistic.cpu_distribution.history_time() / SECONDS_PER_DAY;
        let upper_bound_multiplier = 1.0 + 1.0 / period_in_days;
        let lower_bound_multiplier = (1.0 + 0.001 / period_in_days).powi(-2);

        let mut to_evict = false;
        let mut vpa_recommendation = VPARecommendation {
            pod_id: pod.id,
            new_requested_cpu: pod.requested_cpu,
            new_limit_cpu: pod.limit_cpu,
            new_requested_memory: pod.requested_memory,
            new_limit_memory: pod.limit_memory,
        };

        if (pod.requested_cpu as f64) > upper_bound_multiplier * pod_statistic.cpu_distribution.percentile(DEFAULT_UPPER_BOUND_CPU_PERCENTILE) ||
            (pod.requested_cpu as f64) < lower_bound_multiplier * pod_statistic.cpu_distribution.percentile(DEFAULT_LOWER_BOUND_CPU_PERCENTILE)  {
            vpa_recommendation.new_requested_cpu = pod_statistic.cpu_distribution.percentile(DEFAULT_TARGET_CPU_PERCENTILE) as f32;
            if self.controlled_values == RequestsAndLimits {
                vpa_recommendation.new_limit_cpu = vpa_recommendation.new_requested_cpu *
                    (pod.limit_cpu / pod.requested_cpu);
            }
            to_evict = true;
        }

        if pod.requested_memory > upper_bound_multiplier * pod_statistic.memory_distribution.percentile(DEFAULT_UPPER_BOUND_MEMORY_PERCENTILE) ||
            pod.requested_memory < lower_bound_multiplier * pod_statistic.memory_distribution.percentile(DEFAULT_LOWER_BOUND_MEMORY_PERCENTILE) {
            vpa_recommendation.new_requested_memory = pod_statistic.memory_distribution.percentile(DEFAULT_TARGET_MEMORY_PERCENTILE);
            if self.controlled_values == RequestsAndLimits {
                vpa_recommendation.new_limit_memory = vpa_recommendation.new_requested_memory *
                    (pod.limit_memory / pod.requested_memory);
            }
            to_evict = true;
        }

        if to_evict {
            Some(vpa_recommendation)
        } else {
            None
        }
    }

    fn try_to_apply_recommendation(&mut self, pod: &Pod, node: &Rc<RefCell<Node>>, recommendation: VPARecommendation) -> bool {
        true
    }
}