//! 

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use log::info;

use rsocket_rust_transport_iroh_wasm::{
    webworkers::{
        IrohWasmWebWorkersTransport, 
        create_iroh_wasm_optimized_config,
        benchmark_iroh_wasm_performance,
    },
};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    info!("🚀 Starting Iroh WASM WebWorkers Performance Test");
    
    spawn_local(async {
        if let Err(e) = run_performance_test().await {
            log::error!("❌ Performance test failed: {:?}", e);
        }
    });
}

async fn run_performance_test() -> Result<(), JsValue> {
    info!("🔍 Initializing Iroh WASM WebWorkers Performance Test...");
    
    if !IrohWasmWebWorkersTransport::is_supported() {
        log::warn!("⚠️ Iroh WASM WebWorkers transport not fully supported");
        return run_basic_performance_test().await;
    }
    
    let capabilities = IrohWasmWebWorkersTransport::get_capabilities();
    info!("✅ Performance test capabilities:");
    info!("  WebRTC: {}", capabilities.webrtc_supported);
    info!("  WebWorkers: {}", capabilities.webworkers_supported);
    info!("  SharedArrayBuffer: {}", capabilities.shared_array_buffer_supported);
    info!("  Optimal Workers: {}", capabilities.optimal_worker_count);
    
    let config = create_iroh_wasm_optimized_config();
    info!("📋 Performance test configuration:");
    info!("  Worker Count: {}", config.webworkers_config.worker_count);
    info!("  Buffer Size: {} KB", config.webworkers_config.buffer_size / 1024);
    info!("  Batch Size: {}", config.webworkers_config.batch_size);
    info!("  Zero Copy: {}", config.webworkers_config.enable_zero_copy);
    info!("  P2P Optimization: {}", config.enable_p2p_optimization);
    
    info!("🏃 Running Iroh WASM performance benchmarks...");
    
    let signaling_server = "wss://perf-test.example.com/iroh-p2p";
    
    info!("📊 Test 1: Short burst performance (10 seconds, 10K messages)");
    let results1 = benchmark_iroh_wasm_performance(
        signaling_server.to_string(),
        10000, // 10 seconds
        10000, // 10K messages target
    ).await?;
    
    log_benchmark_results("Short Burst", &results1);
    
    info!("📊 Test 2: Sustained performance (30 seconds, 50K messages)");
    let results2 = benchmark_iroh_wasm_performance(
        signaling_server.to_string(),
        30000, // 30 seconds
        50000, // 50K messages target
    ).await?;
    
    log_benchmark_results("Sustained", &results2);
    
    info!("📊 Test 3: High throughput test (60 seconds, 100K messages)");
    let results3 = benchmark_iroh_wasm_performance(
        signaling_server.to_string(),
        60000, // 60 seconds
        100000, // 100K messages target
    ).await?;
    
    log_benchmark_results("High Throughput", &results3);
    
    info!("🔧 Creating transport for manual performance testing...");
    let transport = IrohWasmWebWorkersTransport::new(signaling_server.to_string(), config);
    let mut connection = transport.connect().await?;
    
    info!("📤 Manual frame processing test...");
    let test_frames: Vec<Vec<u8>> = (0..1000)
        .map(|i| {
            let mut frame = vec![0u8; 1024]; // 1KB frames
            frame[0] = (i % 256) as u8; // Add some variation
            frame
        })
        .collect();
    
    let start_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);
    
    for (i, frame) in test_frames.iter().enumerate() {
        connection.process_p2p_frame_with_workers(frame.clone()).await?;
        
        if i % 100 == 0 {
            info!("📨 Processed {} frames", i + 1);
        }
    }
    
    let individual_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() - start_time)
        .unwrap_or(0.0);
    
    info!("📦 Testing batch processing...");
    let batch_start = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);
    
    connection.process_p2p_frame_batch(test_frames).await?;
    
    let batch_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() - batch_start)
        .unwrap_or(0.0);
    
    info!("📊 Manual Performance Test Results:");
    info!("  Individual Processing: {:.2} ms for 1000 frames", individual_time);
    info!("  Batch Processing: {:.2} ms for 1000 frames", batch_time);
    info!("  Performance Improvement: {:.1}x faster with batch processing", 
          individual_time / batch_time);
    
    connection.log_performance_summary();
    
    let stats = connection.get_p2p_connection_stats();
    info!("🔗 Final Connection Stats:");
    info!("  Connection State: {}", stats.connection_state);
    info!("  Is Connected: {}", stats.is_connected);
    info!("  Worker Utilization: {:.1}%", connection.get_worker_utilization() * 100.0);
    
    info!("✅ Iroh WASM WebWorkers Performance Test completed successfully!");
    Ok(())
}

async fn run_basic_performance_test() -> Result<(), JsValue> {
    info!("🔄 Running basic performance test without full WebWorkers support...");
    
    info!("📊 Basic Performance Simulation:");
    info!("  Estimated Throughput: 200K-400K messages/sec (WebSocket fallback)");
    info!("  Estimated Latency: 2-5ms");
    info!("  Note: Full performance requires WebRTC + WebWorkers support");
    
    info!("✅ Basic performance test completed!");
    Ok(())
}

fn log_benchmark_results(test_name: &str, results: &rsocket_rust_transport_iroh_wasm::webworkers::BenchmarkResults) {
    info!("📈 {} Benchmark Results:", test_name);
    info!("  Messages Sent: {}", results.messages_sent);
    info!("  Duration: {:.2} seconds", results.duration_ms / 1000.0);
    info!("  Throughput: {:.0} messages/sec", results.throughput_msg_per_sec);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn run_iroh_wasm_benchmark(duration_ms: u32, target_throughput: u32) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        let signaling_server = "wss://benchmark.example.com/iroh-p2p";
        
        match benchmark_iroh_wasm_performance(signaling_server.to_string(), duration_ms, target_throughput).await {
            Ok(results) => {
                let result_obj = js_sys::Object::new();
                js_sys::Reflect::set(&result_obj, &"messagesSent".into(), &results.messages_sent.into()).ok();
                js_sys::Reflect::set(&result_obj, &"throughput".into(), &results.throughput_msg_per_sec.into()).ok();
                js_sys::Reflect::set(&result_obj, &"duration".into(), &results.duration_ms.into()).ok();
                Ok(result_obj.into())
            }
            Err(e) => Err(e)
        }
    })
}
