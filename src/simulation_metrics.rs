use std::fs::File;
use std::io::{BufWriter, Error, Write};
use serde::Serialize;

#[derive(Serialize)]
pub struct Metrics {
    pub timestamp: f64,
    pub cpu_load_rate: f64,
    pub cpu_average_load: f64,
    pub memory_load_rate: f64,
    pub memory_average_load: f64,
}

impl Metrics {
    pub fn new(timestamp: f64, cpu_load_rate: f64, cpu_average_load: f64,
               memory_load_rate: f64, memory_average_load: f64) -> Self {
        Self {
            timestamp,
            cpu_load_rate,
            cpu_average_load,
            memory_load_rate,
            memory_average_load
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
        println!("Time: {}, CPU load rate: {}, CPU average load: {}, \
         memory load rate {}, memory average load {}", metrics.timestamp, metrics.cpu_load_rate,
                 metrics.cpu_average_load, metrics.memory_load_rate, metrics.memory_average_load)
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