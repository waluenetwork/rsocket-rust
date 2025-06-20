//! 

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use log::info;

use rsocket_rust_transport_iroh_wasm::{
    IrohWasmClientTransport,
    IrohWasmConfig,
    webworkers::{IrohWasmWebWorkersTransport, create_iroh_wasm_optimized_config},
};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    info!("🚀 Starting Iroh WASM P2P Demo");
    
    spawn_local(async {
        if let Err(e) = run_demo().await {
            log::error!("❌ Demo failed: {:?}", e);
        }
    });
}

async fn run_demo() -> Result<(), JsValue> {
    info!("🔍 Checking Iroh WASM capabilities...");
    
    if !IrohWasmWebWorkersTransport::is_supported() {
        log::warn!("⚠️ Iroh WASM WebWorkers transport not fully supported in this environment");
        return run_basic_demo().await;
    }
    
    let capabilities = IrohWasmWebWorkersTransport::get_capabilities();
    info!("✅ Iroh WASM capabilities detected:");
    info!("  WebRTC: {}", capabilities.webrtc_supported);
    info!("  WebWorkers: {}", capabilities.webworkers_supported);
    info!("  SharedArrayBuffer: {}", capabilities.shared_array_buffer_supported);
    info!("  Optimal Workers: {}", capabilities.optimal_worker_count);
    
    let config = create_iroh_wasm_optimized_config();
    info!("📋 Using optimized configuration:");
    info!("  Worker Count: {}", config.webworkers_config.worker_count);
    info!("  Buffer Size: {} KB", config.webworkers_config.buffer_size / 1024);
    info!("  P2P Optimization: {}", config.enable_p2p_optimization);
    
    let signaling_server = "wss://signaling.example.com/iroh-p2p";
    let transport = IrohWasmWebWorkersTransport::new(signaling_server.to_string(), config);
    
    info!("🔗 Connecting to Iroh P2P network via signaling server: {}", signaling_server);
    
    let mut connection = transport.connect().await?;
    info!("✅ Iroh WASM P2P connection established successfully!");
    
    info!("📤 Testing P2P frame processing with WebWorkers...");
    
    let test_frames = vec![
        vec![0u8; 1024],  // 1KB frame
        vec![1u8; 2048],  // 2KB frame
        vec![2u8; 4096],  // 4KB frame
    ];
    
    for (i, frame) in test_frames.into_iter().enumerate() {
        info!("📨 Processing frame {} ({} bytes)", i + 1, frame.len());
        connection.process_p2p_frame_with_workers(frame).await?;
    }
    
    info!("📦 Testing batch frame processing...");
    let batch_frames: Vec<Vec<u8>> = (0..10)
        .map(|i| vec![i as u8; 1024])
        .collect();
    
    connection.process_p2p_frame_batch(batch_frames).await?;
    
    info!("📊 Performance Summary:");
    connection.log_performance_summary();
    
    let stats = connection.get_p2p_connection_stats();
    info!("🔗 Connection Stats:");
    info!("  Connection State: {}", stats.connection_state);
    info!("  Data Channel State: {}", stats.data_channel_state);
    info!("  Is Connected: {}", stats.is_connected);
    info!("  Worker Utilization: {:.1}%", connection.get_worker_utilization() * 100.0);
    
    info!("✅ Iroh WASM P2P Demo completed successfully!");
    Ok(())
}

async fn run_basic_demo() -> Result<(), JsValue> {
    info!("🔄 Running basic Iroh WASM demo without WebWorkers...");
    
    let config = IrohWasmConfig::default();
    let signaling_server = "wss://signaling.example.com/iroh-p2p";
    
    let transport = IrohWasmClientTransport::new(signaling_server.to_string(), config);
    
    info!("🔗 Connecting to Iroh P2P network...");
    let connection = transport.connect().await?;
    
    info!("✅ Basic Iroh WASM connection established!");
    
    let stats = connection.get_connection_stats();
    info!("🔗 Connection Stats:");
    info!("  Connection State: {}", stats.connection_state);
    info!("  Data Channel State: {}", stats.data_channel_state);
    info!("  Is Connected: {}", stats.is_connected);
    
    info!("✅ Basic Iroh WASM demo completed!");
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    log(&format!("Hello, {}! Welcome to Iroh WASM P2P Demo.", name));
}
