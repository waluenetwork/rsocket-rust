
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_util::{Sink, Stream};



use rsocket_rust::async_trait;
use rsocket_rust::{transport::Transport, Result, error::RSocketError, frame::Frame};

use super::{WebWorkersConfig, PerformanceMonitor, WorkerPool};
use super::wasm_compat::{wasm_spawn, WasmMutex};
use crate::{WebsocketClientTransport, WebsocketConnection};
use rsocket_rust::utils::Writeable;

#[derive(Debug)]
pub struct WebWorkersClientTransport {
    websocket_transport: WebsocketClientTransport,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
    worker_pool: Option<Rc<WorkerPool>>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct WebWorkersConnection {
    websocket_connection: WebsocketConnection,
    worker_pool: Rc<WorkerPool>,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
    frame_cache: HashMap<u32, Frame>,
    next_frame_id: u32,
}

impl WebWorkersClientTransport {
    pub fn new(websocket_url: String, config: WebWorkersConfig) -> Self {
        let websocket_transport = WebsocketClientTransport::from(websocket_url);
        
        let performance_monitor = if config.enable_performance_monitoring {
            Some(PerformanceMonitor::new())
        } else {
            None
        };

        Self {
            websocket_transport,
            config,
            performance_monitor,
            worker_pool: None,
        }
    }

    pub fn with_config(websocket_url: String, config: WebWorkersConfig) -> Self {
        Self::new(websocket_url, config)
    }

    async fn initialize_workers(&mut self) -> Result<()> {
        if self.worker_pool.is_some() {
            return Ok(());
        }

        let worker_pool = WorkerPool::new(self.config.clone()).await
            .map_err(|e| RSocketError::Other(anyhow::anyhow!("Failed to initialize worker pool: {:?}", e)))?;

        self.worker_pool = Some(Rc::new(worker_pool));
        Ok(())
    }

    pub fn get_performance_metrics(&self) -> Option<super::performance::PerformanceMetrics> {
        self.performance_monitor.as_ref().map(|m| m.get_metrics())
    }

    pub fn is_supported() -> bool {
        super::worker::is_webworkers_supported()
    }

    pub fn get_capabilities() -> super::worker::WebWorkersCapabilities {
        super::worker::detect_webworkers_capabilities()
    }
}

#[cfg(target_arch = "wasm32")]
pub trait WasmTransport {
    type Conn: WasmConnection;
    
    async fn connect(self) -> Result<Self::Conn>;
}

#[cfg(target_arch = "wasm32")]
pub trait WasmConnection {
    fn split(self) -> (Box<dyn WasmFrameSink>, Box<dyn WasmFrameStream>);
}

#[cfg(target_arch = "wasm32")]
pub trait WasmFrameSink {
    fn send(&mut self, frame: Frame) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>>>>;
    fn flush(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>>>>;
    fn close(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>>>>;
}

#[cfg(target_arch = "wasm32")]
pub trait WasmFrameStream {
    fn next(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<Result<Frame>>>>>;
}

#[cfg(target_arch = "wasm32")]
impl WasmTransport for WebWorkersClientTransport {
    type Conn = WebWorkersConnection;

    async fn connect(mut self) -> Result<Self::Conn> {
        self.initialize_workers().await?;

        let websocket_connection = self.websocket_transport.connect().await?;

        let worker_pool = self.worker_pool
            .ok_or_else(|| RSocketError::Other(anyhow::anyhow!("Worker pool not initialized")))?;

        let connection = WebWorkersConnection {
            websocket_connection,
            worker_pool,
            config: self.config,
            performance_monitor: self.performance_monitor,
            frame_cache: HashMap::new(),
            next_frame_id: 1,
        };
        
        Ok(connection)
    }
}

// WebWorkers are WASM-only, no non-WASM implementation needed

#[cfg(target_arch = "wasm32")]
impl WebWorkersConnection {
    pub async fn process_frame_with_workers(&mut self, frame: Frame) -> Result<()> {
        let start_time = super::performance::high_precision_timestamp();

        if let Some(monitor) = &mut self.performance_monitor {
            monitor.record_message(frame.len());
        }

        let frame_bytes = frame.bytes().to_vec();
        let result = self.worker_pool.process_frame(frame_bytes).await;
        result
            .map_err(|e| RSocketError::Other(anyhow::anyhow!("Worker processing failed: {:?}", e)))?;

        if let Some(monitor) = &mut self.performance_monitor {
            let latency = super::performance::high_precision_timestamp() - start_time;
            monitor.record_latency(latency);
        }

        Ok(())
    }

    pub async fn process_frame_batch(&mut self, frames: Vec<Frame>) -> Result<()> {
        if frames.is_empty() {
            return Ok(());
        }

        let start_time = super::performance::high_precision_timestamp();

        if let Some(monitor) = &mut self.performance_monitor {
            for frame in &frames {
                monitor.record_message(frame.len());
            }
        }

        let frame_bytes: Vec<Vec<u8>> = frames.into_iter()
            .map(|f| f.bytes().to_vec())
            .collect();

        let result = self.worker_pool.process_frame_batch(frame_bytes).await;
        result
            .map_err(|e| RSocketError::Other(anyhow::anyhow!("Batch processing failed: {:?}", e)))?;

        if let Some(monitor) = &mut self.performance_monitor {
            let latency = super::performance::high_precision_timestamp() - start_time;
            monitor.record_latency(latency);
        }

        Ok(())
    }

    pub async fn get_aggregate_performance_metrics(&self) -> super::performance::PerformanceMetrics {
        self.worker_pool.get_aggregate_performance_metrics().await
    }

    pub fn get_worker_count(&self) -> usize {
        self.worker_pool.get_worker_count()
    }

    pub fn log_performance_summary(&self) {
        if let Some(monitor) = &self.performance_monitor {
            monitor.log_performance_summary();
        }
    }

    pub fn get_worker_utilization(&self) -> f64 {
        self.worker_pool.get_worker_utilization()
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmConnection for WebWorkersConnection {
    fn split(self) -> (Box<dyn WasmFrameSink>, Box<dyn WasmFrameStream>) {
        let (base_sink, base_stream) = self.websocket_connection.split();
        
        let enhanced_sink = WebWorkersWasmFrameSink {
            base_sink: Some(base_sink),
            worker_pool: Rc::clone(&self.worker_pool),
            config: self.config.clone(),
            performance_monitor: self.performance_monitor,
        };

        let enhanced_stream = WebWorkersWasmFrameStream {
            base_stream: Some(base_stream),
            worker_pool: Rc::clone(&self.worker_pool),
            config: self.config.clone(),
        };

        (Box::new(enhanced_sink), Box::new(enhanced_stream))
    }
}

// WebWorkers Connection is WASM-only

#[cfg(target_arch = "wasm32")]
struct WebWorkersWasmFrameSink {
    base_sink: Option<Box<rsocket_rust::transport::FrameSink>>,
    worker_pool: Rc<WorkerPool>,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
}

#[cfg(target_arch = "wasm32")]
struct WebWorkersWasmFrameStream {
    base_stream: Option<Box<rsocket_rust::transport::FrameStream>>,
    worker_pool: Rc<WorkerPool>,
    config: WebWorkersConfig,
}

// WebWorkers Frame Sink/Stream are WASM-only

#[cfg(target_arch = "wasm32")]
impl WasmFrameSink for WebWorkersWasmFrameSink {
    fn send(&mut self, frame: Frame) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>>>> {
        let worker_pool = Rc::clone(&self.worker_pool);
        let frame_bytes = frame.bytes().to_vec();
        
        wasm_spawn(async move {
            if let Err(e) = worker_pool.process_frame(frame_bytes).await {
                web_sys::console::error_1(&format!("Worker processing failed: {:?}", e).into());
            }
        });

        if let Some(mut base_sink) = self.base_sink.take() {
            Box::pin(async move {
                use futures_util::SinkExt;
                base_sink.send(frame).await.map_err(|e| RSocketError::Other(anyhow::anyhow!(e)))
            })
        } else {
            Box::pin(async move {
                Err(RSocketError::Other(anyhow::anyhow!("Base sink not available")))
            })
        }
    }

    fn flush(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>>>> {
        if let Some(mut base_sink) = self.base_sink.take() {
            Box::pin(async move {
                use futures_util::SinkExt;
                base_sink.flush().await.map_err(|e| RSocketError::Other(anyhow::anyhow!(e)))
            })
        } else {
            Box::pin(async move {
                Err(RSocketError::Other(anyhow::anyhow!("Base sink not available")))
            })
        }
    }

    fn close(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>>>> {
        if let Some(mut base_sink) = self.base_sink.take() {
            Box::pin(async move {
                use futures_util::SinkExt;
                base_sink.close().await.map_err(|e| RSocketError::Other(anyhow::anyhow!(e)))
            })
        } else {
            Box::pin(async move {
                Err(RSocketError::Other(anyhow::anyhow!("Base sink not available")))
            })
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmFrameStream for WebWorkersWasmFrameStream {
    fn next(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<Result<Frame>>>>> {
        if let Some(mut base_stream) = self.base_stream.take() {
            Box::pin(async move {
                use futures_util::StreamExt;
                match base_stream.next().await {
                    Some(Ok(frame)) => Some(Ok(frame)),
                    Some(Err(e)) => Some(Err(RSocketError::Other(anyhow::anyhow!(e)))),
                    None => None,
                }
            })
        } else {
            Box::pin(async move { None })
        }
    }
}

// WebWorkers Sink/Stream implementations are WASM-only

impl From<String> for WebWorkersClientTransport {
    fn from(websocket_url: String) -> Self {
        Self::new(websocket_url, WebWorkersConfig::default())
    }
}

impl From<&str> for WebWorkersClientTransport {
    fn from(websocket_url: &str) -> Self {
        Self::from(websocket_url.to_string())
    }
}

pub fn create_optimized_config() -> WebWorkersConfig {
    let capabilities = super::worker::detect_webworkers_capabilities();
    
    WebWorkersConfig {
        worker_count: capabilities.optimal_worker_count,
        buffer_size: if capabilities.shared_array_buffer_supported { 
            2 * 1024 * 1024 // 2MB with SharedArrayBuffer
        } else { 
            512 * 1024 // 512KB fallback
        },
        batch_size: 200, // Larger batches for better throughput
        enable_shared_array_buffer: capabilities.shared_array_buffer_supported,
        enable_performance_monitoring: true,
        enable_zero_copy: capabilities.shared_array_buffer_supported,
        max_concurrent_tasks: 2000,
    }
}

pub fn log_webworkers_info(config: &WebWorkersConfig) {
    let capabilities = super::worker::detect_webworkers_capabilities();
    
    web_sys::console::log_1(&format!(
        "WebWorkers RSocket Transport Configuration:\n\
         - Workers: {}\n\
         - Buffer Size: {} KB\n\
         - Batch Size: {}\n\
         - SharedArrayBuffer: {} (supported: {})\n\
         - Zero-Copy: {}\n\
         - Performance Monitoring: {}\n\
         - Hardware Concurrency: {:?}\n\
         - WebWorkers Supported: {}",
        config.worker_count,
        config.buffer_size / 1024,
        config.batch_size,
        config.enable_shared_array_buffer,
        capabilities.shared_array_buffer_supported,
        config.enable_zero_copy,
        config.enable_performance_monitoring,
        capabilities.hardware_concurrency,
        capabilities.workers_supported
    ).into());
}

#[cfg(target_arch = "wasm32")]
pub async fn benchmark_webworkers_performance(
    websocket_url: String,
    duration_ms: u32,
    target_throughput: u32,
) -> Result<super::performance::BenchmarkResults> {
    use super::WasmTransport;
    
    let config = create_optimized_config();
    let transport = WebWorkersClientTransport::new(websocket_url, config);
    
    let mut connection = transport.connect().await?;
    let mut benchmark = super::performance::create_performance_benchmark();
    
    let test_frame = Frame::new(1, rsocket_rust::frame::Body::RequestResponse(
        rsocket_rust::frame::RequestResponse {
            metadata: None,
            data: Some(vec![0u8; 1024].into()),
        }
    ), 0);
    let mut frames_sent = 0;
    
    while !benchmark.is_complete() && frames_sent < target_throughput {
        let frame_copy = Frame::new(1, rsocket_rust::frame::Body::RequestResponse(
            rsocket_rust::frame::RequestResponse {
                metadata: None,
                data: Some(vec![0u8; 1024].into()),
            }
        ), 0);
        connection.process_frame_with_workers(frame_copy).await?;
        benchmark.record_message();
        frames_sent += 1;
        
        if frames_sent % 1000 == 0 {
            wasm_bindgen_futures::JsFuture::from(
                js_sys::Promise::resolve(&JsValue::from(1))
            ).await.ok();
        }
    }
    
    Ok(benchmark.get_results())
}
