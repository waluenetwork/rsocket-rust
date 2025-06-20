

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub messages_processed: u64,
    pub total_time_ms: f64,
    pub throughput_messages_per_sec: f64,
    pub average_latency_ms: f64,
    pub peak_latency_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug)]
pub struct PerformanceMonitor {
    messages_sent: u64,
    messages_received: u64,
    total_bytes_sent: u64,
    total_bytes_received: u64,
    latency_samples: VecDeque<f64>,
    start_time: f64,
    last_measurement_time: f64,
    successful_operations: u64,
    failed_operations: u64,
    max_latency_samples: usize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        let now = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        Self {
            messages_sent: 0,
            messages_received: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            latency_samples: VecDeque::new(),
            start_time: now,
            last_measurement_time: now,
            successful_operations: 0,
            failed_operations: 0,
            max_latency_samples: 1000,
        }
    }
    
    pub fn record_message_sent(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.total_bytes_sent += bytes as u64;
        self.last_measurement_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
    }
    
    pub fn record_message_received(&mut self, bytes: usize) {
        self.messages_received += 1;
        self.total_bytes_received += bytes as u64;
    }
    
    pub fn record_latency(&mut self, latency_ms: f64) {
        self.latency_samples.push_back(latency_ms);
        
        if self.latency_samples.len() > self.max_latency_samples {
            self.latency_samples.pop_front();
        }
    }
    
    pub fn record_success(&mut self) {
        self.successful_operations += 1;
    }
    
    pub fn record_failure(&mut self) {
        self.failed_operations += 1;
    }
    
    pub fn get_throughput_messages_per_sec(&self) -> f64 {
        let elapsed_time_sec = (self.last_measurement_time - self.start_time) / 1000.0;
        if elapsed_time_sec > 0.0 {
            self.messages_sent as f64 / elapsed_time_sec
        } else {
            0.0
        }
    }
    
    pub fn get_throughput_bytes_per_sec(&self) -> f64 {
        let elapsed_time_sec = (self.last_measurement_time - self.start_time) / 1000.0;
        if elapsed_time_sec > 0.0 {
            self.total_bytes_sent as f64 / elapsed_time_sec
        } else {
            0.0
        }
    }
    
    pub fn get_average_latency(&self) -> f64 {
        if self.latency_samples.is_empty() {
            0.0
        } else {
            self.latency_samples.iter().sum::<f64>() / self.latency_samples.len() as f64
        }
    }
    
    pub fn get_peak_latency(&self) -> f64 {
        self.latency_samples.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(0.0)
    }
    
    pub fn get_success_rate(&self) -> f64 {
        let total_operations = self.successful_operations + self.failed_operations;
        if total_operations > 0 {
            self.successful_operations as f64 / total_operations as f64
        } else {
            0.0
        }
    }
    
    pub fn log_performance_summary(&self) {
        log::info!("🚀 WebWorkers Performance Summary:");
        log::info!("  Messages Sent: {}", self.messages_sent);
        log::info!("  Messages Received: {}", self.messages_received);
        log::info!("  Bytes Sent: {} KB", self.total_bytes_sent / 1024);
        log::info!("  Bytes Received: {} KB", self.total_bytes_received / 1024);
        log::info!("  Throughput: {:.0} msg/sec, {:.0} KB/sec", 
                  self.get_throughput_messages_per_sec(), 
                  self.get_throughput_bytes_per_sec() / 1024.0);
        log::info!("  Average Latency: {:.2} ms", self.get_average_latency());
        log::info!("  Peak Latency: {:.2} ms", self.get_peak_latency());
        log::info!("  Success Rate: {:.1}%", self.get_success_rate() * 100.0);
    }
    
    pub fn reset(&mut self) {
        self.messages_sent = 0;
        self.messages_received = 0;
        self.total_bytes_sent = 0;
        self.total_bytes_received = 0;
        self.latency_samples.clear();
        self.successful_operations = 0;
        self.failed_operations = 0;
        self.start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        self.last_measurement_time = self.start_time;
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_performance_benchmark() -> PerformanceBenchmark {
    PerformanceBenchmark::new()
}

#[derive(Debug)]
pub struct PerformanceBenchmark {
    monitor: PerformanceMonitor,
    target_duration_ms: f64,
    target_message_count: u64,
    is_complete: bool,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            monitor: PerformanceMonitor::new(),
            target_duration_ms: 10000.0, // 10 seconds default
            target_message_count: 10000, // 10K messages default
            is_complete: false,
        }
    }
    
    pub fn with_duration(mut self, duration_ms: f64) -> Self {
        self.target_duration_ms = duration_ms;
        self
    }
    
    pub fn with_message_count(mut self, count: u64) -> Self {
        self.target_message_count = count;
        self
    }
    
    pub fn record_message(&mut self) {
        self.monitor.record_message_sent(1024); // Assume 1KB messages
        self.monitor.record_success();
        
        let elapsed = self.monitor.last_measurement_time - self.monitor.start_time;
        if elapsed >= self.target_duration_ms || self.monitor.messages_sent >= self.target_message_count {
            self.is_complete = true;
        }
    }
    
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
    
    pub fn get_results(&self) -> BenchmarkResults {
        let elapsed_time_ms = self.monitor.last_measurement_time - self.monitor.start_time;
        
        BenchmarkResults {
            messages_processed: self.monitor.messages_sent,
            total_time_ms: elapsed_time_ms,
            throughput_messages_per_sec: self.monitor.get_throughput_messages_per_sec(),
            average_latency_ms: self.monitor.get_average_latency(),
            peak_latency_ms: self.monitor.get_peak_latency(),
            success_rate: self.monitor.get_success_rate(),
        }
    }
}
