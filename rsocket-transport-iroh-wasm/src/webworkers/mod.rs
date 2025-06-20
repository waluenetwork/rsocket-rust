
pub mod iroh_wasm_webworkers;
pub mod performance;
pub mod worker_pool;

pub use iroh_wasm_webworkers::{IrohWasmWebWorkersTransport, IrohWasmWebWorkersConnection};
pub use performance::{IrohWasmPerformanceMonitor, IrohWasmPerformanceMetrics};
pub use worker_pool::{IrohWasmWorkerPool};

#[derive(Debug, Clone)]
pub struct WebWorkersConfig {
    pub worker_count: usize,
    pub buffer_size: usize,
    pub batch_size: usize,
    pub enable_shared_array_buffer: bool,
    pub enable_performance_monitoring: bool,
    pub enable_zero_copy: bool,
    pub max_concurrent_tasks: usize,
}

pub fn detect_webworkers_capabilities() -> WebWorkersCapabilities {
    WebWorkersCapabilities {
        optimal_worker_count: 4,
        max_buffer_size: 1024 * 1024,
        shared_array_buffer_supported: false,
    }
}

#[derive(Debug, Clone)]
pub struct WebWorkersCapabilities {
    pub optimal_worker_count: usize,
    pub max_buffer_size: usize,
    pub shared_array_buffer_supported: bool,
}
use crate::misc::{IrohWasmConfig, detect_iroh_wasm_capabilities};

#[derive(Debug, Clone)]
pub struct IrohWasmWebWorkersConfig {
    pub iroh_config: IrohWasmConfig,
    pub webworkers_config: WebWorkersConfig,
    pub enable_p2p_optimization: bool,
    pub enable_webrtc_fallback: bool,
    pub max_peer_connections: usize,
}

impl Default for IrohWasmWebWorkersConfig {
    fn default() -> Self {
        let capabilities = detect_iroh_wasm_capabilities();
        let webworkers_capabilities = detect_webworkers_capabilities();
        
        Self {
            iroh_config: IrohWasmConfig::default(),
            webworkers_config: WebWorkersConfig {
                worker_count: webworkers_capabilities.optimal_worker_count,
                buffer_size: webworkers_capabilities.max_buffer_size,
                batch_size: 100,
                enable_shared_array_buffer: webworkers_capabilities.shared_array_buffer_supported,
                enable_performance_monitoring: true,
                enable_zero_copy: true,
                max_concurrent_tasks: 1000,
            },
            enable_p2p_optimization: capabilities.webrtc_supported,
            enable_webrtc_fallback: true,
            max_peer_connections: 10,
        }
    }
}

pub fn create_iroh_wasm_optimized_config() -> IrohWasmWebWorkersConfig {
    let capabilities = detect_iroh_wasm_capabilities();
    let webworkers_capabilities = detect_webworkers_capabilities();
    
    IrohWasmWebWorkersConfig {
        iroh_config: IrohWasmConfig {
            enable_webworkers: capabilities.webworkers_supported,
            worker_count: capabilities.optimal_worker_count,
            buffer_size: if capabilities.shared_array_buffer_supported { 
                2 * 1024 * 1024 
            } else { 
                512 * 1024 
            },
            enable_performance_monitoring: true,
            connection_timeout_ms: 15000,
            max_retries: 5,
            ..IrohWasmConfig::default()
        },
        webworkers_config: WebWorkersConfig {
            worker_count: webworkers_capabilities.optimal_worker_count,
            buffer_size: if webworkers_capabilities.shared_array_buffer_supported { 
                2 * 1024 * 1024 
            } else { 
                512 * 1024 
            },
            batch_size: 150,
            enable_shared_array_buffer: webworkers_capabilities.shared_array_buffer_supported,
            enable_performance_monitoring: true,
            enable_zero_copy: webworkers_capabilities.shared_array_buffer_supported,
            max_concurrent_tasks: 1500,
        },
        enable_p2p_optimization: capabilities.webrtc_supported,
        enable_webrtc_fallback: true,
        max_peer_connections: 15,
    }
}

pub async fn benchmark_iroh_wasm_performance(
    signaling_server: String,
    duration_ms: u32,
    target_messages: u32,
) -> Result<BenchmarkResults, wasm_bindgen::JsValue> {
    
    let config = create_iroh_wasm_optimized_config();
    let transport = IrohWasmWebWorkersTransport::new(signaling_server, config);
    
    let mut connection = transport.connect().await?;
    
    let start_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);
    
    let mut message_count = 0;
    while message_count < target_messages {
        let test_frame = vec![0u8; 1024];
        
        if let Err(e) = connection.process_p2p_frame_with_workers(test_frame).await {
            log::warn!("P2P frame processing failed: {:?}", e);
        }
        
        message_count += 1;
        
        if message_count % 1000 == 0 {
            let elapsed = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() - start_time)
                .unwrap_or(0.0);
            
            if elapsed > duration_ms as f64 {
                break;
            }
        }
    }
    
    let elapsed = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() - start_time)
        .unwrap_or(0.0);
    
    Ok(BenchmarkResults {
        messages_sent: message_count,
        duration_ms: elapsed,
        throughput_msg_per_sec: (message_count as f64 / elapsed) * 1000.0,
    })
}

#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub messages_sent: u32,
    pub duration_ms: f64,
    pub throughput_msg_per_sec: f64,
}
