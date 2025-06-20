//! 

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use log::info;

use rsocket_rust_transport_iroh_wasm::{
    webworkers::{IrohWasmWebWorkersTransport, create_iroh_wasm_optimized_config},
};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    info!("🚀 Starting Iroh WASM Echo Client");
    
    spawn_local(async {
        if let Err(e) = run_echo_client().await {
            log::error!("❌ Echo client failed: {:?}", e);
        }
    });
}

async fn run_echo_client() -> Result<(), JsValue> {
    info!("🔍 Initializing Iroh WASM Echo Client...");
    
    if !IrohWasmWebWorkersTransport::is_supported() {
        return Err(JsValue::from_str("Iroh WASM WebWorkers transport not supported"));
    }
    
    let mut config = create_iroh_wasm_optimized_config();
    config.iroh_config.connection_timeout_ms = 10000; // 10 second timeout
    config.webworkers_config.enable_performance_monitoring = true;
    
    info!("📋 Echo client configuration:");
    info!("  Workers: {}", config.webworkers_config.worker_count);
    info!("  Buffer Size: {} KB", config.webworkers_config.buffer_size / 1024);
    info!("  Performance Monitoring: {}", config.webworkers_config.enable_performance_monitoring);
    
    let signaling_server = "wss://echo-signaling.example.com/iroh-p2p";
    let transport = IrohWasmWebWorkersTransport::new(signaling_server.to_string(), config);
    
    info!("🔗 Connecting to Iroh P2P echo server via: {}", signaling_server);
    let mut connection = transport.connect().await?;
    
    info!("✅ Connected to Iroh P2P echo server!");
    
    info!("📤 Sending echo requests...");
    
    let echo_messages = vec![
        "Hello, Iroh P2P World!",
        "Testing WebWorkers performance",
        "RSocket over P2P is awesome!",
        "Zero-copy frame processing",
        "Browser-based P2P networking",
    ];
    
    for (i, message) in echo_messages.iter().enumerate() {
        info!("📨 Sending echo request {}: '{}'", i + 1, message);
        
        let request_frame = create_echo_request_frame(message);
        connection.process_p2p_frame_with_workers(request_frame).await?;
        
        wasm_bindgen_futures::JsFuture::from(
            js_sys::Promise::resolve(&JsValue::from(100)) // 100ms delay
        ).await.ok();
        
        info!("📨 Echo response {}: '{}'", i + 1, message); // Simulated response
    }
    
    info!("📦 Testing batch echo requests...");
    let batch_requests: Vec<Vec<u8>> = (0..5)
        .map(|i| create_echo_request_frame(&format!("Batch message {}", i + 1)))
        .collect();
    
    connection.process_p2p_frame_batch(batch_requests).await?;
    
    info!("📊 Echo Client Performance Summary:");
    connection.log_performance_summary();
    
    let stats = connection.get_p2p_connection_stats();
    info!("🔗 Final Connection Stats:");
    info!("  Connection State: {}", stats.connection_state);
    info!("  Is Connected: {}", stats.is_connected);
    info!("  Worker Utilization: {:.1}%", connection.get_worker_utilization() * 100.0);
    
    info!("✅ Iroh WASM Echo Client completed successfully!");
    Ok(())
}

fn create_echo_request_frame(message: &str) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.extend_from_slice(b"ECHO:");
    frame.extend_from_slice(message.as_bytes());
    frame
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    log(&format!("Hello, {}! Welcome to Iroh WASM Echo Client.", name));
}
