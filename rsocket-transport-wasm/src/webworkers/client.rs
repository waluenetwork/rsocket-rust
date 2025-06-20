
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::cell::RefCell;

use crate::client::WebsocketClientTransport;
use crate::connection::WebsocketConnection;
use rsocket_rust::transport::{FrameSink, FrameStream, Transport, Connection};
use super::{
    WebWorkersConfig, PerformanceMonitor, SharedRingBuffer, WorkerPool,
    wasm_traits::{WasmTransport, WasmConnection, WasmFrameSink, WasmFrameStream},
    is_webworkers_supported, detect_webworkers_capabilities,
};

#[derive(Debug)]
pub struct WebWorkersClientTransport {
    websocket_transport: WebsocketClientTransport,
    config: WebWorkersConfig,
    worker_pool: Option<Rc<RefCell<WorkerPool>>>,
    performance_monitor: Option<PerformanceMonitor>,
}

#[derive(Debug)]
pub struct WebWorkersConnection {
    websocket_connection: WebsocketConnection,
    worker_pool: Rc<RefCell<WorkerPool>>,
    shared_buffer: Option<SharedRingBuffer>,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
}

impl WebWorkersClientTransport {
    pub fn new(url: String, config: WebWorkersConfig) -> Self {
        let websocket_transport = WebsocketClientTransport::from(url);
        let performance_monitor = if config.enable_performance_monitoring {
            Some(PerformanceMonitor::new())
        } else {
            None
        };
        
        Self {
            websocket_transport,
            config,
            worker_pool: None,
            performance_monitor,
        }
    }
    
    async fn initialize_workers(&mut self) -> Result<(), JsValue> {
        if self.worker_pool.is_some() {
            return Ok(());
        }
        
        if !is_webworkers_supported() {
            return Err(JsValue::from_str("WebWorkers not supported"));
        }
        
        let worker_pool = WorkerPool::new(self.config.worker_count)?;
        self.worker_pool = Some(Rc::new(RefCell::new(worker_pool)));
        
        log::info!("✅ Initialized WebWorkers pool with {} workers", self.config.worker_count);
        Ok(())
    }
    
    pub fn is_supported() -> bool {
        is_webworkers_supported()
    }
    
    pub fn get_capabilities() -> super::WebWorkersCapabilities {
        detect_webworkers_capabilities()
    }
    
    pub fn get_performance_metrics(&self) -> Option<&PerformanceMonitor> {
        self.performance_monitor.as_ref()
    }
}

impl WasmTransport for WebWorkersClientTransport {
    type Conn = WebWorkersConnection;
    
    fn connect(mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Conn, JsValue>>>> {
        Box::pin(async move {
            self.initialize_workers().await?;
            
            let websocket_connection = Transport::connect(self.websocket_transport).await
                .map_err(|e| JsValue::from_str(&format!("WebSocket connection failed: {:?}", e)))?;
            
            let worker_pool = self.worker_pool
                .ok_or_else(|| JsValue::from_str("Worker pool not initialized"))?;
            
            let shared_buffer = if self.config.enable_shared_array_buffer && 
                                  super::shared_buffer::is_shared_array_buffer_supported() {
                Some(SharedRingBuffer::new(self.config.buffer_size)?)
            } else {
                None
            };
            
            let connection = WebWorkersConnection {
                websocket_connection,
                worker_pool,
                shared_buffer,
                config: self.config,
                performance_monitor: self.performance_monitor,
            };
            
            log::info!("✅ WebWorkers connection established successfully");
            Ok(connection)
        })
    }
}

impl WebWorkersConnection {
    pub async fn process_frame_with_workers(&mut self, frame: Vec<u8>) -> Result<(), JsValue> {
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        if let Some(monitor) = &mut self.performance_monitor {
            monitor.record_message_sent(frame.len());
        }
        
        if let Some(shared_buffer) = &self.shared_buffer {
            if shared_buffer.write_frame(&frame)? {
                log::debug!("Frame written to shared buffer for zero-copy processing");
            } else {
                self.worker_pool.borrow_mut().process_frame(frame)?;
            }
        } else {
            self.worker_pool.borrow_mut().process_frame(frame)?;
        }
        
        if let Some(monitor) = &mut self.performance_monitor {
            let latency = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() - start_time)
                .unwrap_or(0.0);
            monitor.record_latency(latency);
            monitor.record_success();
        }
        
        Ok(())
    }
    
    pub async fn process_frame_batch(&mut self, frames: Vec<Vec<u8>>) -> Result<(), JsValue> {
        if frames.is_empty() {
            return Ok(());
        }
        
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        self.worker_pool.borrow_mut().process_batch(frames.clone())?;
        
        if let Some(monitor) = &mut self.performance_monitor {
            for frame in &frames {
                monitor.record_message_sent(frame.len());
            }
            
            let latency = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() - start_time)
                .unwrap_or(0.0);
            monitor.record_latency(latency);
            monitor.record_success();
        }
        
        Ok(())
    }
    
    pub fn get_worker_utilization(&self) -> f64 {
        self.worker_pool.borrow().get_worker_utilization()
    }
    
    pub fn get_queue_length(&self) -> usize {
        self.worker_pool.borrow().get_total_queue_length()
    }
    
    pub fn log_performance_summary(&self) {
        if let Some(monitor) = &self.performance_monitor {
            monitor.log_performance_summary();
        }
        
        log::info!("🔧 WebWorkers Stats:");
        log::info!("  Worker Utilization: {:.1}%", self.get_worker_utilization() * 100.0);
        log::info!("  Queue Length: {}", self.get_queue_length());
        
        if let Some(shared_buffer) = &self.shared_buffer {
            log::info!("  Shared Buffer Frames: {}", shared_buffer.frame_count());
            log::info!("  Shared Buffer Available: {} bytes", shared_buffer.available_write_space());
        }
    }
}

impl WasmConnection for WebWorkersConnection {
    fn split(self) -> (Box<dyn WasmFrameSink>, Box<dyn WasmFrameStream>) {
        let (websocket_sink, websocket_stream) = Connection::split(self.websocket_connection);
        
        let enhanced_sink = WebWorkersFrameSink {
            websocket_sink,
            worker_pool: Rc::clone(&self.worker_pool),
            shared_buffer: self.shared_buffer,
            config: self.config.clone(),
            performance_monitor: self.performance_monitor,
        };
        
        let enhanced_stream = WebWorkersFrameStream {
            websocket_stream,
            worker_pool: self.worker_pool,
        };
        
        (Box::new(enhanced_sink), Box::new(enhanced_stream))
    }
}

struct WebWorkersFrameSink {
    websocket_sink: Box<FrameSink>,
    worker_pool: Rc<RefCell<WorkerPool>>,
    shared_buffer: Option<SharedRingBuffer>,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
}

struct WebWorkersFrameStream {
    websocket_stream: Box<FrameStream>,
    worker_pool: Rc<RefCell<WorkerPool>>,
}

impl WasmFrameSink for WebWorkersFrameSink {
    fn send(&mut self, frame: Vec<u8>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), JsValue>> + '_>> {
        let worker_pool = Rc::clone(&self.worker_pool);
        let frame_for_worker = frame.clone();
        
        spawn_local(async move {
            if let Err(e) = worker_pool.borrow_mut().process_frame(frame_for_worker) {
                log::error!("Worker processing failed: {:?}", e);
            }
        });
        
        use rsocket_rust::frame::RequestResponse;
        use futures_util::SinkExt;
        use bytes::Bytes;
        
        let frame_bytes = Bytes::from(frame);
        let rsocket_frame = RequestResponse::builder(0, 0)
            .set_data(frame_bytes)
            .build();
        
        Box::pin(async move {
            self.websocket_sink.send(rsocket_frame).await
                .map_err(|e| JsValue::from_str(&format!("WebSocket send failed: {:?}", e)))
        })
    }
}

impl WasmFrameStream for WebWorkersFrameStream {
    fn next(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<Result<Vec<u8>, JsValue>>> + '_>> {
        use futures_util::StreamExt;
        Box::pin(async move {
            match self.websocket_stream.next().await {
                Some(Ok(rsocket_frame)) => {
                    use rsocket_rust::utils::Writeable;
                    let frame_bytes = rsocket_frame.bytes();
                    
                    if let Err(e) = self.worker_pool.borrow_mut().process_frame(frame_bytes.clone()) {
                        log::warn!("Worker processing of received frame failed: {:?}", e);
                    }
                    Some(Ok(frame_bytes))
                }
                Some(Err(e)) => Some(Err(JsValue::from_str(&format!("WebSocket receive failed: {:?}", e)))),
                None => None,
            }
        })
    }
}

impl From<String> for WebWorkersClientTransport {
    fn from(url: String) -> Self {
        Self::new(url, WebWorkersConfig::default())
    }
}

impl From<&str> for WebWorkersClientTransport {
    fn from(url: &str) -> Self {
        Self::from(url.to_string())
    }
}
