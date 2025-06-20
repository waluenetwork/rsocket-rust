
#[cfg(all(feature = "wasm-only", feature = "webworkers"))]
pub use super::client::{WebWorkersClientTransport, WebWorkersConnection};

#[cfg(all(feature = "wasm-only", feature = "webworkers"))]
pub use super::{WebWorkersConfig, PerformanceMonitor, SharedRingBuffer, WorkerPool, RSocketWorker};

#[cfg(all(feature = "wasm-only", feature = "webworkers"))]
pub use super::wasm_traits::{WasmTransport, WasmConnection, WasmFrameSink, WasmFrameStream};

#[cfg(all(feature = "wasm-only", feature = "webworkers"))]
pub fn create_optimized_webworkers_transport(url: &str) -> WebWorkersClientTransport {
    let capabilities = super::detect_webworkers_capabilities();
    
    let config = WebWorkersConfig {
        worker_count: capabilities.optimal_worker_count,
        buffer_size: capabilities.max_buffer_size,
        batch_size: 150,
        enable_shared_array_buffer: capabilities.shared_array_buffer_supported,
        enable_performance_monitoring: true,
        enable_zero_copy: capabilities.shared_array_buffer_supported,
        max_concurrent_tasks: 1500,
    };
    
    WebWorkersClientTransport::new(url.to_string(), config)
}

#[cfg(not(all(feature = "wasm-only", feature = "webworkers")))]
pub fn create_optimized_webworkers_transport(_url: &str) -> () {
    panic!("WebWorkers functionality requires both 'wasm-only' and 'webworkers' features");
}
