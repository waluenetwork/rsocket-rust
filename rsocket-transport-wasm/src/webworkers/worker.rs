
use wasm_bindgen::prelude::*;
use web_sys::{Worker, MessagePort, DedicatedWorkerGlobalScope};
use std::collections::HashMap;
use std::rc::Rc;
use futures_util::future::join_all;

use super::{WebWorkersConfig, PerformanceMonitor, SharedRingBuffer};

#[derive(Debug)]
pub struct WorkerPool {
    workers: Vec<RSocketWorker>,
    config: WebWorkersConfig,
    next_worker: std::sync::atomic::AtomicUsize,
    performance_monitor: Option<PerformanceMonitor>,
}

#[derive(Debug)]
pub struct RSocketWorker {
    worker: Worker,
    worker_id: usize,
    config: WorkerConfig,
    message_count: std::sync::atomic::AtomicU64,
    is_busy: std::sync::atomic::AtomicBool,
}

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: usize,
    pub max_concurrent_tasks: usize,
    pub enable_performance_monitoring: bool,
    pub buffer_size: usize,
}

impl WorkerPool {
    pub async fn new(config: WebWorkersConfig) -> Result<Self, JsValue> {
        let mut workers = Vec::new();
        
        let worker_script_url = create_worker_script_blob()?;
        
        for i in 0..config.worker_count {
            let worker_config = WorkerConfig {
                worker_id: i,
                max_concurrent_tasks: config.max_concurrent_tasks,
                enable_performance_monitoring: config.enable_performance_monitoring,
                buffer_size: config.buffer_size / config.worker_count,
            };
            
            let worker = RSocketWorker::new(worker_config, &worker_script_url).await?;
            workers.push(worker);
        }
        
        let performance_monitor = if config.enable_performance_monitoring {
            Some(PerformanceMonitor::new())
        } else {
            None
        };

        Ok(Self {
            workers,
            config,
            next_worker: std::sync::atomic::AtomicUsize::new(0),
            performance_monitor,
        })
    }

    pub async fn process_frame(&self, frame: Vec<u8>) -> Result<(), JsValue> {
        let worker_idx = self.next_worker.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.workers.len();
        let worker = &self.workers[worker_idx];
        
        if let Some(monitor) = &self.performance_monitor {
        }
        
        worker.process_frame(frame).await
    }

    pub async fn process_frame_batch(&self, frames: Vec<Vec<u8>>) -> Result<(), JsValue> {
        if frames.is_empty() {
            return Ok(());
        }

        let chunk_size = (frames.len() + self.workers.len() - 1) / self.workers.len();
        let mut tasks = Vec::new();

        for (worker_idx, chunk) in frames.chunks(chunk_size).enumerate() {
            if worker_idx >= self.workers.len() {
                break;
            }
            
            let worker = &self.workers[worker_idx];
            let chunk_vec = chunk.to_vec();
            
            tasks.push(worker.process_frame_batch(chunk_vec));
        }

        let results: Vec<Result<(), JsValue>> = join_all(tasks).await;
        
        for result in results {
            result?;
        }

        Ok(())
    }

    pub async fn get_aggregate_performance_metrics(&self) -> super::performance::PerformanceMetrics {
        let mut total_messages = 0;
        let mut total_bytes = 0;
        let mut total_latency = 0.0;
        let mut worker_count = 0;

        for worker in &self.workers {
            let metrics = worker.get_performance_metrics().await;
            total_messages += metrics.total_messages;
            total_bytes += metrics.total_bytes;
            total_latency += metrics.avg_latency_ms;
            worker_count += 1;
        }

        super::performance::PerformanceMetrics {
            messages_per_second: total_messages as f64, // Simplified calculation
            avg_latency_ms: if worker_count > 0 { total_latency / worker_count as f64 } else { 0.0 },
            total_messages,
            total_bytes,
            worker_utilization: self.get_worker_utilization(),
            ..Default::default()
        }
    }

    pub fn get_worker_utilization(&self) -> f64 {
        let busy_workers = self.workers.iter()
            .filter(|w| w.is_busy.load(std::sync::atomic::Ordering::Relaxed))
            .count();
        
        busy_workers as f64 / self.workers.len() as f64
    }

    pub fn get_worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn shutdown(&self) {
        for worker in &self.workers {
            worker.terminate();
        }
    }
}

impl RSocketWorker {
    pub async fn new(config: WorkerConfig, script_url: &str) -> Result<Self, JsValue> {
        let worker = Worker::new(script_url)?;
        
        let init_message = serde_json::json!({
            "type": "init",
            "config": {
                "worker_id": config.worker_id,
                "max_concurrent_tasks": config.max_concurrent_tasks,
                "enable_performance_monitoring": config.enable_performance_monitoring,
                "buffer_size": config.buffer_size
            }
        });
        
        worker.post_message(&JsValue::from_str(&init_message.to_string()))?;

        Ok(Self {
            worker,
            worker_id: config.worker_id,
            config,
            message_count: std::sync::atomic::AtomicU64::new(0),
            is_busy: std::sync::atomic::AtomicBool::new(false),
        })
    }

    pub async fn process_frame(&self, frame: Vec<u8>) -> Result<(), JsValue> {
        self.is_busy.store(true, std::sync::atomic::Ordering::Relaxed);
        
        let message = serde_json::json!({
            "type": "process_frame",
            "frame": frame,
            "frame_id": self.message_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        });
        
        self.worker.post_message(&JsValue::from_str(&message.to_string()))?;
        
        self.is_busy.store(false, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }

    pub async fn process_frame_batch(&self, frames: Vec<Vec<u8>>) -> Result<(), JsValue> {
        self.is_busy.store(true, std::sync::atomic::Ordering::Relaxed);
        
        let message = serde_json::json!({
            "type": "process_frame_batch",
            "frames": frames,
            "batch_id": self.message_count.fetch_add(frames.len() as u64, std::sync::atomic::Ordering::Relaxed)
        });
        
        self.worker.post_message(&JsValue::from_str(&message.to_string()))?;
        
        self.is_busy.store(false, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }

    pub async fn get_performance_metrics(&self) -> super::performance::PerformanceMetrics {
        super::performance::PerformanceMetrics {
            total_messages: self.message_count.load(std::sync::atomic::Ordering::Relaxed),
            worker_utilization: if self.is_busy.load(std::sync::atomic::Ordering::Relaxed) { 1.0 } else { 0.0 },
            ..Default::default()
        }
    }

    pub fn terminate(&self) {
        self.worker.terminate();
    }
}

pub fn create_worker_script_blob() -> Result<String, JsValue> {
    let script_content = RSOCKET_WORKER_SCRIPT;
    
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&JsValue::from_str(script_content));
    
    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type("application/javascript");
    
    let blob = web_sys::Blob::new_with_str_sequence(&blob_parts)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    
    Ok(url)
}

pub const RSOCKET_WORKER_SCRIPT: &str = r#"
let workerConfig = null;
let performanceMetrics = {
    messagesProcessed: 0,
    bytesProcessed: 0,
    startTime: performance.now(),
    lastUpdate: performance.now()
};

self.onmessage = function(event) {
    const message = JSON.parse(event.data);
    
    switch (message.type) {
        case 'init':
            workerConfig = message.config;
            console.log(`RSocket Worker ${workerConfig.worker_id} initialized`);
            break;
            
        case 'process_frame':
            processFrame(message.frame, message.frame_id);
            break;
            
        case 'process_frame_batch':
            processFrameBatch(message.frames, message.batch_id);
            break;
            
        case 'get_metrics':
            self.postMessage({
                type: 'metrics_response',
                metrics: getPerformanceMetrics()
            });
            break;
            
        default:
            console.warn(`Unknown message type: ${message.type}`);
    }
};

function processFrame(frameData, frameId) {
    const startTime = performance.now();
    
    try {
        
        const frameSize = frameData.length;
        
        let checksum = 0;
        for (let i = 0; i < frameData.length; i++) {
            checksum ^= frameData[i];
        }
        
        performanceMetrics.messagesProcessed++;
        performanceMetrics.bytesProcessed += frameSize;
        
        const processingTime = performance.now() - startTime;
        
        self.postMessage({
            type: 'frame_processed',
            frameId: frameId,
            processingTime: processingTime,
            checksum: checksum
        });
        
    } catch (error) {
        self.postMessage({
            type: 'frame_error',
            frameId: frameId,
            error: error.message
        });
    }
}

function processFrameBatch(frames, batchId) {
    const startTime = performance.now();
    const results = [];
    
    try {
        for (let i = 0; i < frames.length; i++) {
            const frameData = frames[i];
            const frameSize = frameData.length;
            
            let checksum = 0;
            for (let j = 0; j < frameData.length; j++) {
                checksum ^= frameData[j];
            }
            
            results.push({
                frameIndex: i,
                checksum: checksum,
                size: frameSize
            });
            
            performanceMetrics.messagesProcessed++;
            performanceMetrics.bytesProcessed += frameSize;
        }
        
        const processingTime = performance.now() - startTime;
        
        self.postMessage({
            type: 'batch_processed',
            batchId: batchId,
            results: results,
            processingTime: processingTime,
            framesProcessed: frames.length
        });
        
    } catch (error) {
        self.postMessage({
            type: 'batch_error',
            batchId: batchId,
            error: error.message
        });
    }
}

function getPerformanceMetrics() {
    const now = performance.now();
    const elapsedSeconds = (now - performanceMetrics.startTime) / 1000;
    
    return {
        messagesProcessed: performanceMetrics.messagesProcessed,
        bytesProcessed: performanceMetrics.bytesProcessed,
        messagesPerSecond: elapsedSeconds > 0 ? performanceMetrics.messagesProcessed / elapsedSeconds : 0,
        bytesPerSecond: elapsedSeconds > 0 ? performanceMetrics.bytesProcessed / elapsedSeconds : 0,
        workerId: workerConfig ? workerConfig.worker_id : -1,
        uptime: elapsedSeconds
    };
}

console.log('RSocket WebWorker started');
"#;

pub fn is_webworkers_supported() -> bool {
    use wasm_bindgen::JsValue;
    use js_sys::global;
    
    let global = global();
    js_sys::Reflect::has(&global, &JsValue::from_str("Worker")).unwrap_or(false)
}

pub fn get_optimal_worker_count() -> usize {
    super::navigator_hardware_concurrency().unwrap_or(4).min(8) // Cap at 8 workers
}

pub fn detect_webworkers_capabilities() -> WebWorkersCapabilities {
    WebWorkersCapabilities {
        workers_supported: is_webworkers_supported(),
        shared_array_buffer_supported: super::is_shared_array_buffer_supported(),
        hardware_concurrency: super::navigator_hardware_concurrency(),
        optimal_worker_count: get_optimal_worker_count(),
    }
}

#[derive(Debug, Clone)]
pub struct WebWorkersCapabilities {
    pub workers_supported: bool,
    pub shared_array_buffer_supported: bool,
    pub hardware_concurrency: Option<usize>,
    pub optimal_worker_count: usize,
}
