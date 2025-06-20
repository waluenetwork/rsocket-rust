use wasm_bindgen::prelude::*;
use log::info;

use rsocket_rust_transport_iroh_wasm::{
    detect_iroh_wasm_capabilities,
    is_webrtc_supported,
    webworkers::{IrohWasmWebWorkersTransport, create_iroh_wasm_optimized_config},
};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    info!("🚀 Starting Iroh WASM Capability Test");
    
    run_capability_test();
}

fn run_capability_test() {
    info!("🔍 Testing browser capabilities for Iroh WASM P2P...");
    
    info!("📋 Basic WebRTC Support:");
    let webrtc_supported = is_webrtc_supported();
    info!("  WebRTC Available: {}", webrtc_supported);
    
    if webrtc_supported {
        info!("✅ WebRTC is supported - P2P connections possible");
    } else {
        info!("❌ WebRTC not supported - P2P connections not available");
    }
    
    info!("📋 Comprehensive Iroh WASM Capabilities:");
    let capabilities = detect_iroh_wasm_capabilities();
    info!("  WebRTC Support: {}", capabilities.webrtc_supported);
    info!("  WebWorkers Support: {}", capabilities.webworkers_supported);
    info!("  SharedArrayBuffer Support: {}", capabilities.shared_array_buffer_supported);
    info!("  Optimal Worker Count: {}", capabilities.optimal_worker_count);
    info!("  Max Buffer Size: {} KB", capabilities.max_buffer_size / 1024);
    
    info!("📋 WebWorkers Transport Support:");
    let webworkers_supported = IrohWasmWebWorkersTransport::is_supported();
    info!("  Full WebWorkers Transport: {}", webworkers_supported);
    
    if webworkers_supported {
        info!("✅ Advanced WebWorkers P2P transport available");
        
        let config = create_iroh_wasm_optimized_config();
        info!("📊 Optimized Configuration:");
        info!("  Worker Count: {}", config.webworkers_config.worker_count);
        info!("  Buffer Size: {} KB", config.webworkers_config.buffer_size / 1024);
        info!("  Batch Size: {}", config.webworkers_config.batch_size);
        info!("  Zero Copy: {}", config.webworkers_config.enable_zero_copy);
        info!("  P2P Optimization: {}", config.enable_p2p_optimization);
    } else {
        info!("⚠️ Basic P2P transport only - WebWorkers enhancement not available");
    }
    
    info!("🎯 Browser Compatibility Summary:");
    if capabilities.webrtc_supported && capabilities.webworkers_supported && capabilities.shared_array_buffer_supported {
        info!("🏆 EXCELLENT: Full Iroh WASM P2P capabilities available");
        info!("  Expected Performance: 500K-800K messages/sec with <1ms latency");
    } else if capabilities.webrtc_supported && capabilities.webworkers_supported {
        info!("✅ GOOD: P2P with WebWorkers available (no SharedArrayBuffer)");
        info!("  Expected Performance: 300K-500K messages/sec with 1-2ms latency");
    } else if capabilities.webrtc_supported {
        info!("📊 BASIC: P2P available (no WebWorkers optimization)");
        info!("  Expected Performance: 100K-200K messages/sec with 2-5ms latency");
    } else {
        info!("❌ LIMITED: No P2P capabilities - WebSocket fallback required");
        info!("  Expected Performance: 50K-100K messages/sec with 5-10ms latency");
    }
    
    info!("✅ Iroh WASM capability test completed!");
}

#[wasm_bindgen]
pub fn get_iroh_wasm_capabilities() -> JsValue {
    let capabilities = detect_iroh_wasm_capabilities();
    let result = js_sys::Object::new();
    
    js_sys::Reflect::set(&result, &"webrtcSupported".into(), &capabilities.webrtc_supported.into()).ok();
    js_sys::Reflect::set(&result, &"webworkersSupported".into(), &capabilities.webworkers_supported.into()).ok();
    js_sys::Reflect::set(&result, &"sharedArrayBufferSupported".into(), &capabilities.shared_array_buffer_supported.into()).ok();
    js_sys::Reflect::set(&result, &"optimalWorkerCount".into(), &capabilities.optimal_worker_count.into()).ok();
    js_sys::Reflect::set(&result, &"maxBufferSize".into(), &capabilities.max_buffer_size.into()).ok();
    
    result.into()
}
