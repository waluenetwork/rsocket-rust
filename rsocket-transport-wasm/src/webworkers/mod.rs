//! 

pub mod client;
pub mod performance;
pub mod shared_buffer;
pub mod worker;
pub mod wasm_traits;

#[cfg(feature = "wasm-only")]
pub mod simple_wasm;

pub use client::{WebWorkersClientTransport, WebWorkersConnection};
pub use performance::{PerformanceMonitor, BenchmarkResults};
pub use shared_buffer::SharedRingBuffer;
pub use worker::{RSocketWorker, WorkerPool};
pub use wasm_traits::{WasmTransport, WasmConnection, WasmFrameSink, WasmFrameStream};

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

impl Default for WebWorkersConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            buffer_size: 1024 * 1024, // 1MB
            batch_size: 100,
            enable_shared_array_buffer: true,
            enable_performance_monitoring: true,
            enable_zero_copy: true,
            max_concurrent_tasks: 1000,
        }
    }
}

pub fn is_webworkers_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = ["self"], js_name = Worker)]
            type Worker;
            
            #[wasm_bindgen(method, js_name = "postMessage")]
            fn post_message(this: &Worker, message: &JsValue);
        }
        
        js_sys::eval("typeof Worker !== 'undefined'")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn detect_webworkers_capabilities() -> WebWorkersCapabilities {
    #[cfg(target_arch = "wasm32")]
    {
        let webworkers_supported = is_webworkers_supported();
        let shared_array_buffer_supported = js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        
        let navigator_cores = web_sys::window()
            .map(|w| w.navigator().hardware_concurrency())
            .unwrap_or(4.0) as usize;
        
        let optimal_worker_count = if webworkers_supported {
            std::cmp::max(2, std::cmp::min(navigator_cores, 8))
        } else {
            1
        };
        
        WebWorkersCapabilities {
            webworkers_supported,
            shared_array_buffer_supported,
            optimal_worker_count,
            max_buffer_size: if shared_array_buffer_supported { 4 * 1024 * 1024 } else { 1024 * 1024 },
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        WebWorkersCapabilities {
            webworkers_supported: false,
            shared_array_buffer_supported: false,
            optimal_worker_count: 1,
            max_buffer_size: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebWorkersCapabilities {
    pub webworkers_supported: bool,
    pub shared_array_buffer_supported: bool,
    pub optimal_worker_count: usize,
    pub max_buffer_size: usize,
}
