use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;


use crate::{
    client::IrohWasmClientTransport,
    connection::IrohWasmConnection,
};
use super::{IrohWasmWebWorkersConfig, IrohWasmWorkerPool, IrohWasmPerformanceMonitor};

#[derive(Debug)]
pub struct IrohWasmWebWorkersTransport {
    signaling_server: String,
    config: IrohWasmWebWorkersConfig,
    worker_pool: Option<Rc<IrohWasmWorkerPool>>,
    performance_monitor: Option<IrohWasmPerformanceMonitor>,
}

#[derive(Debug)]
pub struct IrohWasmWebWorkersConnection {
    iroh_connection: IrohWasmConnection,
    worker_pool: Rc<IrohWasmWorkerPool>,
    config: IrohWasmWebWorkersConfig,
    performance_monitor: Option<IrohWasmPerformanceMonitor>,
    frame_cache: Rc<RefCell<HashMap<u32, Vec<u8>>>>,
    next_frame_id: Rc<RefCell<u32>>,
}

impl IrohWasmWebWorkersTransport {
    pub fn new(signaling_server: String, config: IrohWasmWebWorkersConfig) -> Self {
        let performance_monitor = if config.iroh_config.enable_performance_monitoring {
            Some(IrohWasmPerformanceMonitor::new())
        } else {
            None
        };

        Self {
            signaling_server,
            config,
            worker_pool: None,
            performance_monitor,
        }
    }

    async fn initialize_workers(&mut self) -> Result<(), JsValue> {
        if self.worker_pool.is_some() {
            return Ok(());
        }

        let worker_pool = IrohWasmWorkerPool::new(self.config.clone()).await?;
        self.worker_pool = Some(Rc::new(worker_pool));
        
        log::info!("✅ Initialized Iroh WASM WebWorkers pool with {} workers", 
                  self.config.webworkers_config.worker_count);
        
        Ok(())
    }

    pub fn get_performance_metrics(&self) -> Option<super::performance::IrohWasmPerformanceMetrics> {
        self.performance_monitor.as_ref().map(|m| m.get_metrics())
    }

    pub fn is_supported() -> bool {
        crate::misc::is_webrtc_supported() && 
        crate::misc::detect_iroh_wasm_capabilities().webworkers_supported
    }

    pub fn get_capabilities() -> crate::misc::IrohWasmCapabilities {
        crate::misc::detect_iroh_wasm_capabilities()
    }
}

impl IrohWasmWebWorkersTransport {
    pub async fn connect(mut self) -> Result<IrohWasmWebWorkersConnection, JsValue> {
        self.initialize_workers().await?;

        let iroh_transport = IrohWasmClientTransport::new(
            self.signaling_server.clone(), 
            self.config.iroh_config.clone()
        );
        
        let iroh_connection = iroh_transport.connect().await?;

        let worker_pool = self.worker_pool
            .ok_or_else(|| JsValue::from_str("Worker pool not initialized"))?;

        let connection = IrohWasmWebWorkersConnection {
            iroh_connection,
            worker_pool,
            config: self.config,
            performance_monitor: self.performance_monitor,
            frame_cache: Rc::new(RefCell::new(HashMap::new())),
            next_frame_id: Rc::new(RefCell::new(1)),
        };

        log::info!("✅ Iroh WASM WebWorkers connection established successfully");
        Ok(connection)
    }
}

impl IrohWasmWebWorkersConnection {
    pub async fn process_p2p_frame_with_workers(&mut self, frame: Vec<u8>) -> Result<(), JsValue> {
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        if let Some(monitor) = &mut self.performance_monitor {
            monitor.record_p2p_message(frame.len());
        }

        let result = self.worker_pool.process_p2p_frame(frame).await;
        result?;

        if let Some(monitor) = &mut self.performance_monitor {
            let latency = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() - start_time)
                .unwrap_or(0.0);
            monitor.record_p2p_latency(latency);
        }

        Ok(())
    }

    pub async fn process_p2p_frame_batch(&mut self, frames: Vec<Vec<u8>>) -> Result<(), JsValue> {
        if frames.is_empty() {
            return Ok(());
        }

        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        if let Some(monitor) = &mut self.performance_monitor {
            for frame in &frames {
                monitor.record_p2p_message(frame.len());
            }
        }

        let result = self.worker_pool.process_p2p_frame_batch(frames).await;
        result?;

        if let Some(monitor) = &mut self.performance_monitor {
            let latency = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() - start_time)
                .unwrap_or(0.0);
            monitor.record_p2p_latency(latency);
        }

        Ok(())
    }

    pub fn get_p2p_connection_stats(&self) -> crate::connection::IrohWasmConnectionStats {
        self.iroh_connection.get_connection_stats()
    }

    pub fn get_worker_utilization(&self) -> f64 {
        self.worker_pool.get_worker_utilization()
    }

    pub fn log_performance_summary(&self) {
        if let Some(monitor) = &self.performance_monitor {
            monitor.log_p2p_performance_summary();
        }
    }
}

impl IrohWasmWebWorkersConnection {
    pub fn split(self) -> (IrohWasmWebWorkersFrameSink, IrohWasmWebWorkersFrameStream) {
        let enhanced_sink = IrohWasmWebWorkersFrameSink {
            worker_pool: Rc::clone(&self.worker_pool),
            config: self.config.clone(),
            performance_monitor: self.performance_monitor.clone(),
        };

        let enhanced_stream = IrohWasmWebWorkersFrameStream {
            worker_pool: Rc::clone(&self.worker_pool),
            config: self.config,
            frame_cache: self.frame_cache,
            next_frame_id: self.next_frame_id,
        };

        (enhanced_sink, enhanced_stream)
    }
}

pub struct IrohWasmWebWorkersFrameSink {
    worker_pool: Rc<IrohWasmWorkerPool>,
    config: IrohWasmWebWorkersConfig,
    performance_monitor: Option<IrohWasmPerformanceMonitor>,
}

pub struct IrohWasmWebWorkersFrameStream {
    worker_pool: Rc<IrohWasmWorkerPool>,
    config: IrohWasmWebWorkersConfig,
    frame_cache: Rc<RefCell<HashMap<u32, Vec<u8>>>>,
    next_frame_id: Rc<RefCell<u32>>,
}

impl IrohWasmWebWorkersFrameSink {
    pub async fn send(&mut self, frame: Vec<u8>) -> Result<(), JsValue> {
        let worker_pool = Rc::clone(&self.worker_pool);
        
        let frame_for_worker = frame.clone();
        spawn_local(async move {
            if let Err(e) = worker_pool.process_p2p_frame(frame_for_worker).await {
                web_sys::console::error_1(&format!("P2P worker processing failed: {:?}", e).into());
            }
        });

        log::debug!("📤 Sent {} bytes via Iroh WASM WebWorkers sink", frame.len());
        Ok(())
    }
}

impl IrohWasmWebWorkersFrameStream {
    pub async fn next(&mut self) -> Option<Result<Vec<u8>, JsValue>> {
        let frame_id = {
            let mut id = self.next_frame_id.borrow_mut();
            *id += 1;
            *id
        };
        
        let simulated_frame = vec![0u8; 1024]; // Simulate 1KB frame
        self.frame_cache.borrow_mut().insert(frame_id, simulated_frame.clone());
        
        log::debug!("📨 Received {} bytes via Iroh WASM WebWorkers stream", simulated_frame.len());
        Some(Ok(simulated_frame))
    }
}

impl From<String> for IrohWasmWebWorkersTransport {
    fn from(signaling_server: String) -> Self {
        Self::new(signaling_server, IrohWasmWebWorkersConfig::default())
    }
}

impl From<&str> for IrohWasmWebWorkersTransport {
    fn from(signaling_server: &str) -> Self {
        Self::from(signaling_server.to_string())
    }
}
