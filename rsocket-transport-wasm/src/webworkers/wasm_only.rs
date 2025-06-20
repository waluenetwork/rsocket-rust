
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent};
use js_sys::{Uint8Array, ArrayBuffer};

use super::{WebWorkersConfig, PerformanceMonitor, WorkerPool};
use super::wasm_traits::{WasmTransport, WasmConnection, WasmFrameSink, WasmFrameStream, WasmFrame};

#[derive(Debug)]
pub struct WasmOnlyWebWorkersTransport {
    websocket_url: String,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
    worker_pool: Option<Rc<WorkerPool>>,
}

#[derive(Debug)]
pub struct WasmOnlyWebWorkersConnection {
    websocket: WebSocket,
    worker_pool: Rc<WorkerPool>,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
    frame_cache: Rc<RefCell<HashMap<u32, WasmFrame>>>,
    next_frame_id: Rc<RefCell<u32>>,
}

impl WasmOnlyWebWorkersTransport {
    pub fn new(websocket_url: String, config: WebWorkersConfig) -> Self {
        let performance_monitor = if config.enable_performance_monitoring {
            Some(PerformanceMonitor::new())
        } else {
            None
        };

        Self {
            websocket_url,
            config,
            performance_monitor,
            worker_pool: None,
        }
    }

    pub fn with_config(websocket_url: String, config: WebWorkersConfig) -> Self {
        Self::new(websocket_url, config)
    }

    async fn initialize_workers(&mut self) -> Result<(), JsValue> {
        if self.worker_pool.is_some() {
            return Ok(());
        }

        let worker_pool = WorkerPool::new(self.config.clone()).await?;
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

impl WasmTransport for WasmOnlyWebWorkersTransport {
    type Conn = WasmOnlyWebWorkersConnection;

    fn connect(mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Conn, JsValue>>>> {
        Box::pin(async move {
            self.initialize_workers().await?;

            let websocket = WebSocket::new(&self.websocket_url)?;
            websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);

            let worker_pool = self.worker_pool
                .ok_or_else(|| JsValue::from_str("Worker pool not initialized"))?;

            let connection = WasmOnlyWebWorkersConnection {
                websocket,
                worker_pool,
                config: self.config,
                performance_monitor: self.performance_monitor,
                frame_cache: Rc::new(RefCell::new(HashMap::new())),
                next_frame_id: Rc::new(RefCell::new(1)),
            };

            Ok(connection)
        })
    }
}

impl WasmOnlyWebWorkersConnection {
    pub async fn process_frame_with_workers(&mut self, frame: WasmFrame) -> Result<(), JsValue> {
        let start_time = super::performance::high_precision_timestamp();

        if let Some(monitor) = &mut self.performance_monitor {
            monitor.record_message(frame.data.len());
        }

        let result = self.worker_pool.process_frame(frame.data).await;
        result?;

        if let Some(monitor) = &mut self.performance_monitor {
            let latency = super::performance::high_precision_timestamp() - start_time;
            monitor.record_latency(latency);
        }

        Ok(())
    }

    pub async fn process_frame_batch(&mut self, frames: Vec<WasmFrame>) -> Result<(), JsValue> {
        if frames.is_empty() {
            return Ok(());
        }

        let start_time = super::performance::high_precision_timestamp();

        if let Some(monitor) = &mut self.performance_monitor {
            for frame in &frames {
                monitor.record_message(frame.data.len());
            }
        }

        let frame_bytes: Vec<Vec<u8>> = frames.into_iter()
            .map(|f| f.data)
            .collect();

        let result = self.worker_pool.process_frame_batch(frame_bytes).await;
        result?;

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

impl WasmConnection for WasmOnlyWebWorkersConnection {
    fn split(self) -> (Box<dyn WasmFrameSink>, Box<dyn WasmFrameStream>) {
        let websocket_clone = self.websocket.clone();
        
        let enhanced_sink = WasmOnlyFrameSink {
            websocket: self.websocket,
            worker_pool: Rc::clone(&self.worker_pool),
            config: self.config.clone(),
            performance_monitor: self.performance_monitor,
        };

        let enhanced_stream = WasmOnlyFrameStream {
            websocket: websocket_clone,
            worker_pool: Rc::clone(&self.worker_pool),
            config: self.config,
            frame_cache: self.frame_cache,
            next_frame_id: self.next_frame_id,
        };

        (Box::new(enhanced_sink), Box::new(enhanced_stream))
    }
}

struct WasmOnlyFrameSink {
    websocket: WebSocket,
    worker_pool: Rc<WorkerPool>,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
}

struct WasmOnlyFrameStream {
    websocket: WebSocket,
    worker_pool: Rc<WorkerPool>,
    config: WebWorkersConfig,
    frame_cache: Rc<RefCell<HashMap<u32, WasmFrame>>>,
    next_frame_id: Rc<RefCell<u32>>,
}

impl WasmFrameSink for WasmOnlyFrameSink {
    fn send(&mut self, frame: Vec<u8>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), JsValue>>>> {
        let websocket = self.websocket.clone();
        let worker_pool = Rc::clone(&self.worker_pool);
        
        Box::pin(async move {
            let frame_for_worker = frame.clone();
            spawn_local(async move {
                if let Err(e) = worker_pool.process_frame(frame_for_worker).await {
                    web_sys::console::error_1(&format!("Worker processing failed: {:?}", e).into());
                }
            });

            let array = Uint8Array::new_with_length(frame.len() as u32);
            array.copy_from(&frame);
            websocket.send_with_array_buffer(&array.buffer())?;
            
            Ok(())
        })
    }
}

impl WasmFrameStream for WasmOnlyFrameStream {
    fn next(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<Result<Vec<u8>, JsValue>>>>> {
        Box::pin(async move { None })
    }
}

impl From<String> for WasmOnlyWebWorkersTransport {
    fn from(websocket_url: String) -> Self {
        Self::new(websocket_url, WebWorkersConfig::default())
    }
}

impl From<&str> for WasmOnlyWebWorkersTransport {
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

pub async fn benchmark_wasm_only_performance(
    websocket_url: String,
    duration_ms: u32,
    target_throughput: u32,
) -> Result<super::performance::BenchmarkResults, JsValue> {
    let config = create_optimized_config();
    let transport = WasmOnlyWebWorkersTransport::new(websocket_url, config);
    
    let mut connection = transport.connect().await?;
    let mut benchmark = super::performance::create_performance_benchmark();
    
    let test_frame = WasmFrame::new(vec![0u8; 1024]);
    let mut frames_sent = 0;
    
    while !benchmark.is_complete() && frames_sent < target_throughput {
        let frame_copy = WasmFrame::new(vec![0u8; 1024]);
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
