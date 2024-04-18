use std::fs::File;
use std::io::{BufWriter, Error, Write};
use serde::Serialize;

#[derive(Serialize)]
pub struct Metrics {
    pub timestamp: f64,
    pub average_cpu_allocated: f64,
    pub average_memory_allocated: f64,
    pub cpu_allocated_load_rate: f64,
    pub memory_allocated_load_rate: f64,
    pub average_cpu_used: f64,
    pub average_memory_used: f64,
    pub cpu_used_load_rate: f64,
    pub memory_used_load_rate: f64,
}

impl Metrics {
    pub fn new(timestamp: f64, average_cpu_allocated: f64, average_memory_allocated: f64,
               cpu_allocated_load_rate: f64, memory_allocated_load_rate: f64,
               average_cpu_used: f64, average_memory_used: f64,
               cpu_used_load_rate: f64, memory_used_load_rate: f64) -> Self {
        Self {
            timestamp,
            average_cpu_allocated,
            average_memory_allocated,
            cpu_allocated_load_rate,
            memory_allocated_load_rate,
            average_cpu_used,
            average_memory_used,
            cpu_used_load_rate,
            memory_used_load_rate,
        }
    }
}

pub trait MetricsLogger {
    fn snapshot_period(&self) -> f64;
    fn log_metrics(&mut self, metrics: Metrics);
    fn save_log(&mut self, path: &str) -> Result<(), std::io::Error>;
}

pub struct EmptyMetricsLogger {}

impl MetricsLogger for EmptyMetricsLogger {
    fn snapshot_period(&self) -> f64 {
        return -1.0;
    }

    fn log_metrics(&mut self, metrics: Metrics) {}

    fn save_log(&mut self, path: &str) -> Result<(), Error> {
        Ok(())
    }
}

pub struct StdoutMetricsLogger {
    snapshot_period: f64,
}

impl StdoutMetricsLogger {
    pub fn new(snapshot_period: f64) -> Self {
        Self {
            snapshot_period
        }
    }
}

impl MetricsLogger for StdoutMetricsLogger {
    fn snapshot_period(&self) -> f64 {
        self.snapshot_period
    }

    fn log_metrics(&mut self, metrics: Metrics) {
        println!("Time: {}, allocated CPU load rate: {}, allocated CPU average load: {}, \
         allocated memory load rate {}, allocated memory average load {}, \
         used CPU load rate: {}, used CPU average load: {}, \
         used memory load rate {}, used memory average load {}",
                 metrics.timestamp,
                 metrics.cpu_allocated_load_rate, metrics.average_cpu_allocated,
                 metrics.memory_allocated_load_rate, metrics.average_memory_allocated,
                 metrics.cpu_used_load_rate, metrics.average_cpu_used,
                 metrics.memory_used_load_rate, metrics.average_memory_used,)
    }

    fn save_log(&mut self, path: &str) -> Result<(), Error> {
        Ok(())
    }
}

pub struct FileMetricsLogger {
    snapshot_period: f64,
    metrics_history: Vec<Metrics>,
}

impl FileMetricsLogger {
    pub fn new(snapshot_period: f64) -> Self {
        Self {
            snapshot_period,
            metrics_history: Vec::default(),
        }
    }
}

impl MetricsLogger for FileMetricsLogger {
    fn snapshot_period(&self) -> f64 {
        self.snapshot_period
    }

    fn log_metrics(&mut self, metrics: Metrics) {
        self.metrics_history.push(metrics);
    }

    fn save_log(&mut self, path: &str) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer(&mut writer, &self.metrics_history)?;
        writer.flush()
    }
}