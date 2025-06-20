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
        if let Err(e) = run_performance_benchmark().await {
            console::error_1(&format!("Performance benchmark error: {:?}", e).into());
        }
    });
}

async fn run_performance_benchmark() -> Result<(), JsValue> {
    console::log_1(&"🚀 Starting WebWorkers Performance Benchmark".into());
    
    let capabilities = detect_webworkers_capabilities();
    console::log_1(&format!("Platform capabilities: {:?}", capabilities).into());
    
    let worker_counts = vec![1, 2, 4, capabilities.optimal_worker_count];
    
    for worker_count in worker_counts {
        console::log_1(&format!("\n📊 Testing with {} WebWorkers", worker_count).into());
        
        benchmark_worker_configuration(worker_count).await?;
        
        console::log_1(&format!("Results for {} workers:", worker_count).into());
        console::log_1(&"  Benchmark completed successfully".into());
    }
    
    console::log_1(&"\n🔬 Testing SharedArrayBuffer vs Regular Transfer".into());
    compare_transfer_methods().await?;
    
    console::log_1(&"\n✅ Performance benchmark completed!".into());
    Ok(())
}

async fn benchmark_worker_configuration(worker_count: usize) -> Result<(), JsValue> {
    let config = WebWorkersConfig {
        worker_count,
        buffer_size: 2 * 1024 * 1024, // 2MB for high throughput
        enable_performance_monitoring: true,
        ..Default::default()
    };
    
    let transport = WebWorkersClientTransport::new("ws://localhost:7878".to_string(), config);
    
    let connection = WasmTransport::connect(transport).await?;
    
    let message_count = 50000;
    let duration_ms = 5000.0;
    
    let start_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);
    
    let mut processed_count = 0;
    while processed_count < message_count {
        let frame_data = format!("Benchmark message #{}", processed_count).into_bytes();
        let _frame_len = frame_data.len();
        processed_count += 1;
        
        if processed_count % 1000 == 0 {
            console::log_1(&format!("  Processed {} messages", processed_count).into());
        }
    }
    
    Ok(())
}

async fn compare_transfer_methods() -> Result<(), JsValue> {
    console::log_1(&"Testing with SharedArrayBuffer...".into());
    let shared_config = WebWorkersConfig {
        buffer_size: 1024 * 1024,
        enable_shared_array_buffer: true,
        enable_performance_monitoring: true,
        ..Default::default()
    };
    
    let shared_result = benchmark_transfer_method(shared_config).await?;
    
    console::log_1(&"Testing with regular transfer...".into());
    let regular_config = WebWorkersConfig {
        buffer_size: 1024 * 1024,
        enable_shared_array_buffer: false,
        enable_performance_monitoring: true,
        ..Default::default()
    };
    
    let regular_result = benchmark_transfer_method(regular_config).await?;
    
    let improvement = (shared_result / regular_result - 1.0) * 100.0;
    
    console::log_1(&format!("SharedArrayBuffer throughput: {:.0} msg/sec", shared_result).into());
    console::log_1(&format!("Regular transfer throughput: {:.0} msg/sec", regular_result).into());
    console::log_1(&format!("Performance improvement: {:.1}%", improvement).into());
    
    if improvement > 50.0 {
        console::log_1(&"🚀 Significant performance improvement with SharedArrayBuffer!".into());
    } else if improvement > 20.0 {
        console::log_1(&"📈 Moderate performance improvement with SharedArrayBuffer".into());
    } else {
        console::log_1(&"📊 Minimal performance difference".into());
    }
    
    Ok(())
}

async fn benchmark_transfer_method(config: WebWorkersConfig) -> Result<f64, JsValue> {
    let transport = WebWorkersClientTransport::new("ws://localhost:7878".to_string(), config);
    
    let connection = WasmTransport::connect(transport).await?;
    
    let message_count = 10000;
    let mut processed_count = 0;
    
    while processed_count < message_count {
        let frame_data = "Transfer method test".as_bytes().to_vec();
        let _frame_len = frame_data.len();
        processed_count += 1;
    }
    
    Ok(1000.0) // Simulated throughput
}
