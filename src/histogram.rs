use std::cmp::{max, min};

#[derive(Clone)]
pub struct Histogram {
    bucket_weight: Vec<u64>,
    total_weight: u64,

    bucket_size: f64,
    min_bucket: usize,
    max_bucket: usize,

    start_time: f64,
    last_sample_time: f64,
}

const NUM_BUCKETS: usize = 100;

impl Histogram {
    pub fn new(max_value: f64) -> Self {
        Self {
            bucket_weight: vec![0; NUM_BUCKETS as usize],
            total_weight: 0,
            bucket_size: max_value / (NUM_BUCKETS as f64),
            min_bucket: NUM_BUCKETS - 1,
            max_bucket: 0,
            start_time: 0.0,
            last_sample_time: 0.0,
        }
    }

    pub fn min(&self) -> f64 {
        if self.bucket_weight[self.min_bucket] == 0 {
            return -1.0;
        } else {
            self.get_bucket_start(self.min_bucket)
        }
    }

    pub fn max(&self) -> f64 {
        if self.bucket_weight[self.max_bucket] == 0 {
            return -1.0;
        } else {
            self.get_bucket_start(self.max_bucket)
        }
    }

    pub fn percentile(&self, percentile: f64) -> f64 {
        if self.bucket_weight[self.max_bucket] == 0 {
            return -1.0;
        }
        let mut partial_sum = 0;
        let threshold = percentile * (self.total_weight as f64);
        let mut bucket = self.min_bucket;
        while bucket < self.max_bucket {
            partial_sum += self.bucket_weight[bucket];
            if partial_sum as f64 >= threshold {
                break
            }
            bucket += 1;
        }
        if bucket + 1 < NUM_BUCKETS {
            self.get_bucket_start(bucket + 1)
        } else {
            self.get_bucket_start(bucket)
        }
    }

    pub fn add_sample(&mut self, value: f64, weight: u64, time: f64) {

        let bucket = self.find_bucket(value);
        self.bucket_weight[bucket] += weight;
        self.total_weight += weight;
        self.min_bucket = min(self.min_bucket, bucket);
        self.max_bucket = max(self.max_bucket, bucket);

        if self.start_time == 0.0 {
            self.start_time = time;
        }
        self.last_sample_time = self.last_sample_time.max(time);
    }

    pub fn history_time(&self) -> f64 {
        self.last_sample_time - self.start_time
    }

    fn find_bucket(&self, value: f64) -> usize {
        let bucket = (value / self.bucket_size).floor() as usize;
        if bucket >= NUM_BUCKETS {
            NUM_BUCKETS - 1
        } else {
            bucket
        }
    }

    fn get_bucket_start(&self, bucket: usize) -> f64 {
        self.bucket_size * (bucket as f64)
    }

}