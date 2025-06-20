use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct IrohWasmPerformanceMetrics {
    pub p2p_messages_sent: u64,
    pub p2p_messages_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub average_latency_ms: f64,
    pub peak_latency_ms: f64,
    pub throughput_messages_per_sec: f64,
    pub throughput_bytes_per_sec: f64,
    pub connection_success_rate: f64,
    pub worker_utilization: f64,
}

#[derive(Debug, Clone)]
pub struct IrohWasmPerformanceMonitor {
    p2p_messages_sent: u64,
    p2p_messages_received: u64,
    total_bytes_sent: u64,
    total_bytes_received: u64,
    latency_samples: VecDeque<f64>,
    start_time: f64,
    last_measurement_time: f64,
    connection_attempts: u64,
    successful_connections: u64,
    max_latency_samples: usize,
}

impl IrohWasmPerformanceMonitor {
    pub fn new() -> Self {
        let now = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        Self {
            p2p_messages_sent: 0,
            p2p_messages_received: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            latency_samples: VecDeque::new(),
            start_time: now,
            last_measurement_time: now,
            connection_attempts: 0,
            successful_connections: 0,
            max_latency_samples: 1000,
        }
    }

    pub fn record_p2p_message(&mut self, bytes: usize) {
        self.p2p_messages_sent += 1;
        self.total_bytes_sent += bytes as u64;
        self.last_measurement_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
    }

    pub fn record_p2p_received(&mut self, bytes: usize) {
        self.p2p_messages_received += 1;
        self.total_bytes_received += bytes as u64;
    }

    pub fn record_p2p_latency(&mut self, latency_ms: f64) {
        self.latency_samples.push_back(latency_ms);
        
        if self.latency_samples.len() > self.max_latency_samples {
            self.latency_samples.pop_front();
        }
    }

    pub fn record_connection_attempt(&mut self, successful: bool) {
        self.connection_attempts += 1;
        if successful {
            self.successful_connections += 1;
        }
    }

    pub fn get_metrics(&self) -> IrohWasmPerformanceMetrics {
        let elapsed_time_sec = (self.last_measurement_time - self.start_time) / 1000.0;
        
        let average_latency = if !self.latency_samples.is_empty() {
            self.latency_samples.iter().sum::<f64>() / self.latency_samples.len() as f64
        } else {
            0.0
        };

        let peak_latency = self.latency_samples.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(0.0);

        let throughput_messages_per_sec = if elapsed_time_sec > 0.0 {
            self.p2p_messages_sent as f64 / elapsed_time_sec
        } else {
            0.0
        };

        let throughput_bytes_per_sec = if elapsed_time_sec > 0.0 {
            self.total_bytes_sent as f64 / elapsed_time_sec
        } else {
            0.0
        };

        let connection_success_rate = if self.connection_attempts > 0 {
            self.successful_connections as f64 / self.connection_attempts as f64
        } else {
            0.0
        };

        IrohWasmPerformanceMetrics {
            p2p_messages_sent: self.p2p_messages_sent,
            p2p_messages_received: self.p2p_messages_received,
            total_bytes_sent: self.total_bytes_sent,
            total_bytes_received: self.total_bytes_received,
            average_latency_ms: average_latency,
            peak_latency_ms: peak_latency,
            throughput_messages_per_sec,
            throughput_bytes_per_sec,
            connection_success_rate,
            worker_utilization: 0.0, // Will be updated by worker pool
        }
    }

    pub fn log_p2p_performance_summary(&self) {
        let metrics = self.get_metrics();
        
        log::info!("🚀 Iroh WASM P2P Performance Summary:");
        log::info!("  Messages Sent: {}", metrics.p2p_messages_sent);
        log::info!("  Messages Received: {}", metrics.p2p_messages_received);
        log::info!("  Bytes Sent: {} KB", metrics.total_bytes_sent / 1024);
        log::info!("  Bytes Received: {} KB", metrics.total_bytes_received / 1024);
        log::info!("  Average Latency: {:.2} ms", metrics.average_latency_ms);
        log::info!("  Peak Latency: {:.2} ms", metrics.peak_latency_ms);
        log::info!("  Throughput: {:.0} msg/sec, {:.0} KB/sec", 
                  metrics.throughput_messages_per_sec, 
                  metrics.throughput_bytes_per_sec / 1024.0);
        log::info!("  Connection Success Rate: {:.1}%", metrics.connection_success_rate * 100.0);
        log::info!("  Worker Utilization: {:.1}%", metrics.worker_utilization * 100.0);
    }

    pub fn reset(&mut self) {
        self.p2p_messages_sent = 0;
        self.p2p_messages_received = 0;
        self.total_bytes_sent = 0;
        self.total_bytes_received = 0;
        self.latency_samples.clear();
        self.connection_attempts = 0;
        self.successful_connections = 0;
        self.start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        self.last_measurement_time = self.start_time;
    }
}

impl Default for IrohWasmPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
