
use wasm_bindgen::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub messages_per_second: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub total_messages: u64,
    pub total_bytes: u64,
    pub bytes_per_second: f64,
    pub worker_utilization: f64,
    pub buffer_utilization: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            messages_per_second: 0.0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            total_messages: 0,
            total_bytes: 0,
            bytes_per_second: 0.0,
            worker_utilization: 0.0,
            buffer_utilization: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct PerformanceMonitor {
    start_time: f64,
    last_update: f64,
    message_count: u64,
    byte_count: u64,
    latency_samples: VecDeque<f64>,
    max_samples: usize,
    update_interval_ms: f64,
    current_metrics: PerformanceMetrics,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        let now = high_precision_timestamp();
        Self {
            start_time: now,
            last_update: now,
            message_count: 0,
            byte_count: 0,
            latency_samples: VecDeque::new(),
            max_samples: 1000, // Keep last 1000 samples for percentile calculation
            update_interval_ms: 1000.0, // Update metrics every second
            current_metrics: PerformanceMetrics::default(),
        }
    }

    pub fn record_message(&mut self, size_bytes: usize) {
        self.message_count += 1;
        self.byte_count += size_bytes as u64;
        
        let now = high_precision_timestamp();
        if now - self.last_update >= self.update_interval_ms {
            self.update_metrics();
        }
    }

    pub fn record_latency(&mut self, latency_ms: f64) {
        self.latency_samples.push_back(latency_ms);
        
        if self.latency_samples.len() > self.max_samples {
            self.latency_samples.pop_front();
        }
    }

    fn update_metrics(&mut self) {
        let now = high_precision_timestamp();
        let elapsed_ms = now - self.last_update;
        let elapsed_secs = elapsed_ms / 1000.0;
        
        if elapsed_secs > 0.0 {
            let messages_since_update = self.message_count;
            let bytes_since_update = self.byte_count;
            
            self.current_metrics.messages_per_second = messages_since_update as f64 / elapsed_secs;
            self.current_metrics.bytes_per_second = bytes_since_update as f64 / elapsed_secs;
        }

        self.current_metrics.total_messages = self.message_count;
        self.current_metrics.total_bytes = self.byte_count;

        if !self.latency_samples.is_empty() {
            let mut sorted_samples: Vec<f64> = self.latency_samples.iter().cloned().collect();
            sorted_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

            self.current_metrics.avg_latency_ms = 
                sorted_samples.iter().sum::<f64>() / sorted_samples.len() as f64;

            let len = sorted_samples.len();
            self.current_metrics.p95_latency_ms = sorted_samples[(len * 95 / 100).min(len - 1)];
            self.current_metrics.p99_latency_ms = sorted_samples[(len * 99 / 100).min(len - 1)];
        }

        self.last_update = now;
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.current_metrics.clone()
    }

    pub fn update_worker_utilization(&mut self, utilization: f64) {
        self.current_metrics.worker_utilization = utilization;
    }

    pub fn update_buffer_utilization(&mut self, utilization: f64) {
        self.current_metrics.buffer_utilization = utilization;
    }

    pub fn log_performance_summary(&self) {
        let metrics = &self.current_metrics;
        web_sys::console::log_1(&format!(
            "WebWorkers RSocket Performance Summary:\n\
             - Messages/sec: {:.0}\n\
             - Bytes/sec: {:.0} KB\n\
             - Avg Latency: {:.2}ms\n\
             - P95 Latency: {:.2}ms\n\
             - P99 Latency: {:.2}ms\n\
             - Total Messages: {}\n\
             - Worker Utilization: {:.1}%\n\
             - Buffer Utilization: {:.1}%",
            metrics.messages_per_second,
            metrics.bytes_per_second / 1024.0,
            metrics.avg_latency_ms,
            metrics.p95_latency_ms,
            metrics.p99_latency_ms,
            metrics.total_messages,
            metrics.worker_utilization * 100.0,
            metrics.buffer_utilization * 100.0
        ).into());
    }

    pub fn is_performance_target_met(&self, target_msgs_per_sec: f64, target_latency_ms: f64) -> bool {
        let metrics = &self.current_metrics;
        metrics.messages_per_second >= target_msgs_per_sec && 
        metrics.avg_latency_ms <= target_latency_ms
    }

    pub fn get_performance_grade(&self) -> char {
        let metrics = &self.current_metrics;
        
        if metrics.messages_per_second >= 800_000.0 && metrics.avg_latency_ms <= 0.5 {
            'A' // Excellent: 800K+ msg/sec, <0.5ms latency
        } else if metrics.messages_per_second >= 500_000.0 && metrics.avg_latency_ms <= 1.0 {
            'B' // Good: 500K+ msg/sec, <1ms latency
        } else if metrics.messages_per_second >= 200_000.0 && metrics.avg_latency_ms <= 2.0 {
            'C' // Fair: 200K+ msg/sec, <2ms latency
        } else if metrics.messages_per_second >= 100_000.0 && metrics.avg_latency_ms <= 5.0 {
            'D' // Poor: 100K+ msg/sec, <5ms latency
        } else {
            'F' // Failing: Below minimum thresholds
        }
    }
}

pub fn high_precision_timestamp() -> f64 {
    use web_sys::window;
    
    window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or_else(|| {
            js_sys::Date::now()
        })
}

pub fn create_performance_benchmark() -> PerformanceBenchmark {
    PerformanceBenchmark::new()
}

#[derive(Debug)]
pub struct PerformanceBenchmark {
    start_time: f64,
    message_count: u32,
    target_messages: u32,
    target_duration_ms: f64,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            start_time: high_precision_timestamp(),
            message_count: 0,
            target_messages: 100_000, // 100K messages
            target_duration_ms: 1000.0, // 1 second
        }
    }

    pub fn record_message(&mut self) {
        self.message_count += 1;
    }

    pub fn is_complete(&self) -> bool {
        self.message_count >= self.target_messages || self.elapsed_ms() >= self.target_duration_ms
    }

    pub fn elapsed_ms(&self) -> f64 {
        high_precision_timestamp() - self.start_time
    }

    pub fn get_throughput(&self) -> f64 {
        let elapsed_secs = self.elapsed_ms() / 1000.0;
        if elapsed_secs > 0.0 {
            self.message_count as f64 / elapsed_secs
        } else {
            0.0
        }
    }

    pub fn get_results(&self) -> BenchmarkResults {
        BenchmarkResults {
            messages_processed: self.message_count,
            elapsed_ms: self.elapsed_ms(),
            messages_per_second: self.get_throughput(),
            target_achieved: self.get_throughput() >= 800_000.0, // 800K msg/sec target
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub messages_processed: u32,
    pub elapsed_ms: f64,
    pub messages_per_second: f64,
    pub target_achieved: bool,
}
