//! 

#[cfg(feature = "webworkers")]
mod client;
#[cfg(feature = "webworkers")]
mod worker;
#[cfg(feature = "webworkers")]
mod shared_buffer;
#[cfg(feature = "webworkers")]
mod performance;
#[cfg(feature = "webworkers")]
mod wasm_compat;
#[cfg(feature = "webworkers")]
mod wasm_traits;
#[cfg(feature = "webworkers")]
mod wasm_only;
#[cfg(feature = "webworkers")]
pub mod simple_wasm;

#[cfg(feature = "webworkers")]
pub use client::WebWorkersClientTransport;
#[cfg(feature = "webworkers")]
pub use worker::{RSocketWorker, WorkerConfig, WorkerPool};
#[cfg(feature = "webworkers")]
pub use shared_buffer::{SharedRingBuffer, BufferConfig};
#[cfg(feature = "webworkers")]
pub use performance::PerformanceMonitor;
#[cfg(feature = "webworkers")]
pub use wasm_traits::{WasmTransport, WasmConnection, WasmFrameSink, WasmFrameStream, WasmFrame};
#[cfg(feature = "webworkers")]
pub use wasm_only::{WasmOnlyWebWorkersTransport, WasmOnlyWebWorkersConnection, create_optimized_config, benchmark_wasm_only_performance};
#[cfg(feature = "webworkers")]
pub use simple_wasm::{SimpleWasmWebWorkersClient, SimpleWasmConnection, create_simple_wasm_config, benchmark_simple_wasm_performance};

#[derive(Debug, Clone)]
#[cfg(feature = "webworkers")]
pub struct WebWorkersConfig {
    pub worker_count: usize,
    pub buffer_size: usize,
    pub batch_size: usize,
    pub enable_shared_array_buffer: bool,
    pub enable_performance_monitoring: bool,
    pub enable_zero_copy: bool,
    pub max_concurrent_tasks: usize,
}

#[cfg(feature = "webworkers")]
impl Default for WebWorkersConfig {
    fn default() -> Self {
        Self {
            worker_count: navigator_hardware_concurrency().unwrap_or(4),
            buffer_size: 1024 * 1024, // 1MB
            batch_size: 100,
            enable_shared_array_buffer: is_shared_array_buffer_supported(),
            enable_performance_monitoring: false,
            enable_zero_copy: true,
            max_concurrent_tasks: 1000,
        }
    }
}

#[cfg(feature = "webworkers")]
fn navigator_hardware_concurrency() -> Option<usize> {
    use web_sys::window;
    window()
        .map(|w| w.navigator().hardware_concurrency() as usize)
}

#[cfg(feature = "webworkers")]
fn is_shared_array_buffer_supported() -> bool {
    use wasm_bindgen::JsValue;
    use js_sys::global;
    
    let global = global();
    js_sys::Reflect::has(&global, &JsValue::from_str("SharedArrayBuffer"))
        .unwrap_or(false)
}
