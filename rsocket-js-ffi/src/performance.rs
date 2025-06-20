use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::JsResult;

#[napi(object)]
pub struct JsPerformanceMetrics {
    start_time: Instant,
    request_count: Arc<AtomicU64>,
    response_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    total_latency_ms: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    bytes_received: Arc<AtomicU64>,
}

#[napi]
impl JsPerformanceMetrics {
    #[napi(constructor)]
    pub fn new() -> JsResult<Self> {
        Ok(JsPerformanceMetrics {
            start_time: Instant::now(),
            request_count: Arc::new(AtomicU64::new(0)),
            response_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            total_latency_ms: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
        })
    }
    
    #[napi]
    pub fn record_request(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }
    
    #[napi]
    pub fn record_response(&self, latency_ms: f64) {
        self.response_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms as u64, Ordering::Relaxed);
    }
    
    #[napi]
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    #[napi]
    pub fn record_bytes_sent(&self, bytes: f64) {
        self.bytes_sent.fetch_add(bytes as u64, Ordering::Relaxed);
    }
    
    #[napi]
    pub fn record_bytes_received(&self, bytes: f64) {
        self.bytes_received.fetch_add(bytes as u64, Ordering::Relaxed);
    }
    
    #[napi]
    pub fn get_request_count(&self) -> f64 {
        self.request_count.load(Ordering::Relaxed) as f64
    }
    
    #[napi]
    pub fn get_response_count(&self) -> f64 {
        self.response_count.load(Ordering::Relaxed) as f64
    }
    
    #[napi]
    pub fn get_error_count(&self) -> f64 {
        self.error_count.load(Ordering::Relaxed) as f64
    }
    
    #[napi]
    pub fn get_average_latency_ms(&self) -> f64 {
        let response_count = self.response_count.load(Ordering::Relaxed);
        if response_count == 0 {
            0.0
        } else {
            let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
            total_latency as f64 / response_count as f64
        }
    }
    
    #[napi]
    pub fn get_throughput_rps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            0.0
        } else {
            let response_count = self.response_count.load(Ordering::Relaxed);
            response_count as f64 / elapsed
        }
    }
    
    #[napi]
    pub fn get_error_rate(&self) -> f64 {
        let total_requests = self.request_count.load(Ordering::Relaxed);
        if total_requests == 0 {
            0.0
        } else {
            let error_count = self.error_count.load(Ordering::Relaxed);
            error_count as f64 / total_requests as f64
        }
    }
    
    #[napi]
    pub fn get_bytes_sent(&self) -> f64 {
        self.bytes_sent.load(Ordering::Relaxed) as f64
    }
    
    #[napi]
    pub fn get_bytes_received(&self) -> f64 {
        self.bytes_received.load(Ordering::Relaxed) as f64
    }
    
    #[napi]
    pub fn get_bandwidth_mbps(&self) -> HashMap<String, f64> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let mut result = HashMap::new();
        
        if elapsed > 0.0 {
            let bytes_sent = self.bytes_sent.load(Ordering::Relaxed) as f64;
            let bytes_received = self.bytes_received.load(Ordering::Relaxed) as f64;
            
            result.insert("sent_mbps".to_string(), (bytes_sent * 8.0) / (elapsed * 1_000_000.0));
            result.insert("received_mbps".to_string(), (bytes_received * 8.0) / (elapsed * 1_000_000.0));
            result.insert("total_mbps".to_string(), ((bytes_sent + bytes_received) * 8.0) / (elapsed * 1_000_000.0));
        } else {
            result.insert("sent_mbps".to_string(), 0.0);
            result.insert("received_mbps".to_string(), 0.0);
            result.insert("total_mbps".to_string(), 0.0);
        }
        
        result
    }
    
    #[napi]
    pub fn get_summary(&self) -> HashMap<String, f64> {
        let mut summary = HashMap::new();
        
        summary.insert("request_count".to_string(), self.get_request_count() as f64);
        summary.insert("response_count".to_string(), self.get_response_count() as f64);
        summary.insert("error_count".to_string(), self.get_error_count() as f64);
        summary.insert("average_latency_ms".to_string(), self.get_average_latency_ms());
        summary.insert("throughput_rps".to_string(), self.get_throughput_rps());
        summary.insert("error_rate".to_string(), self.get_error_rate());
        summary.insert("bytes_sent".to_string(), self.get_bytes_sent() as f64);
        summary.insert("bytes_received".to_string(), self.get_bytes_received() as f64);
        
        let bandwidth = self.get_bandwidth_mbps();
        for (key, value) in bandwidth {
            summary.insert(key, value);
        }
        
        summary
    }
    
    #[napi]
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.request_count.store(0, Ordering::Relaxed);
        self.response_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.total_latency_ms.store(0, Ordering::Relaxed);
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.bytes_received.store(0, Ordering::Relaxed);
    }
    
    #[napi]
    pub fn get_uptime_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}
