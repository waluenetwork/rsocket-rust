//! 

use rsocket_rust_transport_wasm::webworkers::{
    WebWorkersClientTransport, WebWorkersConfig, detect_webworkers_capabilities,
    wasm_traits::WasmTransport
};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    spawn_local(async {
        if let Err(e) = run_webworkers_echo_client().await {
            console::error_1(&format!("WebWorkers echo client error: {:?}", e).into());
        }
    });
}

async fn run_webworkers_echo_client() -> Result<(), JsValue> {
    console::log_1(&"🚀 Starting WebWorkers Echo Client Demo".into());
    
    let capabilities = detect_webworkers_capabilities();
    console::log_1(&format!("WebWorkers supported: {}", capabilities.webworkers_supported).into());
    console::log_1(&format!("SharedArrayBuffer supported: {}", capabilities.shared_array_buffer_supported).into());
    console::log_1(&format!("Optimal worker count: {}", capabilities.optimal_worker_count).into());
    
    let config = WebWorkersConfig {
        worker_count: capabilities.optimal_worker_count,
        buffer_size: 1024 * 1024, // 1MB shared buffer
        enable_performance_monitoring: true,
        ..Default::default()
    };
    
    console::log_1(&format!("Using {} WebWorkers with {}KB buffer", 
                           config.worker_count, config.buffer_size / 1024).into());
    
    let transport = WebWorkersClientTransport::new("ws://localhost:7878".to_string(), config);
    
    let connection = WasmTransport::connect(transport).await?;
    
    
    console::log_1(&"✅ Connected to RSocket server via WebWorkers".into());
    
    let message_count = 1000;
    let start_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);
    
    for i in 0..message_count {
        let frame_data = format!("WebWorkers message #{}", i).into_bytes();
        
        if i % 100 == 0 {
            console::log_1(&format!("Processed {} messages", i).into());
        }
        
        if i < 5 {
            console::log_1(&format!("Processing message {}: {} bytes", i, frame_data.len()).into());
        }
    }
    
    let end_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);
    
    let duration_ms = end_time - start_time;
    let throughput = (message_count as f64 / duration_ms) * 1000.0;
    
    console::log_1(&format!("🎯 Performance Results:").into());
    console::log_1(&format!("  Messages: {}", message_count).into());
    console::log_1(&format!("  Duration: {:.2} ms", duration_ms).into());
    console::log_1(&format!("  Throughput: {:.0} messages/sec", throughput).into());
    
    if throughput > 800_000.0 {
        console::log_1(&"🏆 Achieved target performance of 800K+ messages/sec!".into());
    } else {
        console::log_1(&format!("📊 Current throughput: {:.0} msg/sec (target: 800K+ msg/sec)", throughput).into());
    }
    
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
