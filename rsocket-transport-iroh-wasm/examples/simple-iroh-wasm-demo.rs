use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use log::info;

use rsocket_rust_transport_iroh_wasm::{
    IrohWasmClientTransport, 
    IrohWasmConfig,
    detect_iroh_wasm_capabilities,
    is_webrtc_supported,
};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    info!("🚀 Starting Simple Iroh WASM P2P Demo");
    
    spawn_local(async {
        if let Err(e) = run_simple_demo().await {
            log::error!("❌ Simple demo failed: {:?}", e);
        }
    });
}

async fn run_simple_demo() -> Result<(), JsValue> {
    info!("🔍 Checking Iroh WASM capabilities...");
    
    if !is_webrtc_supported() {
        log::warn!("⚠️ WebRTC not supported in this browser");
        return Ok(());
    }
    
    let capabilities = detect_iroh_wasm_capabilities();
    info!("✅ Iroh WASM capabilities:");
    info!("  WebRTC: {}", capabilities.webrtc_supported);
    info!("  WebWorkers: {}", capabilities.webworkers_supported);
    info!("  SharedArrayBuffer: {}", capabilities.shared_array_buffer_supported);
    info!("  Optimal Workers: {}", capabilities.optimal_worker_count);
    
    let config = IrohWasmConfig::default();
    let signaling_server = "wss://demo.example.com/iroh-p2p";
    
    info!("🔗 Creating Iroh WASM P2P transport...");
    let transport = IrohWasmClientTransport::new(signaling_server.to_string(), config);
    
    info!("📡 Attempting P2P connection...");
    match transport.connect().await {
        Ok(connection) => {
            info!("✅ P2P connection established successfully!");
            
            let stats = connection.get_connection_stats();
            info!("📊 Connection stats:");
            info!("  State: {}", stats.connection_state);
            info!("  Connected: {}", stats.is_connected);
            
            info!("🎯 Simple Iroh WASM P2P demo completed successfully!");
        }
        Err(e) => {
            log::warn!("⚠️ P2P connection failed (expected in demo): {:?}", e);
            info!("💡 This is normal - real P2P requires signaling server setup");
        }
    }
    
    Ok(())
}
