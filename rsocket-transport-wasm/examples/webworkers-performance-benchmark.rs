//! 

use rsocket_rust_transport_wasm::webworkers::{
    WebWorkersClientTransport, WebWorkersConfig, detect_webworkers_capabilities,
    PerformanceBenchmark, create_performance_benchmark
};
use rsocket_rust::{prelude::*, Result};
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

async fn run_performance_benchmark() -> Result<()> {
    console::log_1(&"🚀 Starting WebWorkers Performance Benchmark".into());
    
    let capabilities = detect_webworkers_capabilities();
    console::log_1(&format!("Platform capabilities: {:?}", capabilities).into());
    
    let worker_counts = vec![1, 2, 4, capabilities.optimal_worker_count];
    
    for worker_count in worker_counts {
        console::log_1(&format!("\n📊 Testing with {} WebWorkers", worker_count).into());
        
        let result = benchmark_worker_configuration(worker_count).await?;
        
        console::log_1(&format!("Results for {} workers:", worker_count).into());
        console::log_1(&format!("  Throughput: {:.0} msg/sec", result.throughput_messages_per_sec).into());
        console::log_1(&format!("  Average Latency: {:.2} ms", result.average_latency_ms).into());
        console::log_1(&format!("  Peak Latency: {:.2} ms", result.peak_latency_ms).into());
        console::log_1(&format!("  Success Rate: {:.1}%", result.success_rate * 100.0).into());
        
        if result.throughput_messages_per_sec > 1_000_000.0 {
            console::log_1(&"🏆 EXCELLENT: >1M messages/sec".into());
        } else if result.throughput_messages_per_sec > 800_000.0 {
            console::log_1(&"✅ TARGET ACHIEVED: >800K messages/sec".into());
        } else if result.throughput_messages_per_sec > 500_000.0 {
            console::log_1(&"📈 GOOD: >500K messages/sec".into());
        } else {
            console::log_1(&"📊 BASELINE: <500K messages/sec".into());
        }
    }
    
    console::log_1(&"\n🔬 Testing SharedArrayBuffer vs Regular Transfer".into());
    await compare_transfer_methods().await?;
    
    console::log_1(&"\n✅ Performance benchmark completed!".into());
    Ok(())
}

async fn benchmark_worker_configuration(worker_count: usize) -> Result<rsocket_rust_transport_wasm::webworkers::BenchmarkResults> {
    let config = WebWorkersConfig::default()
        .with_worker_count(worker_count)
        .with_shared_buffer_size(2 * 1024 * 1024) // 2MB for high throughput
        .with_performance_monitoring(true);
    
    let transport = WebWorkersClientTransport::new("ws://localhost:7878", config)?;
    
    let client = RSocketFactory::connect()
        .transport(transport)
        .start()
        .await?;
    
    let mut benchmark = create_performance_benchmark()
        .with_duration(5000.0) // 5 seconds
        .with_message_count(50000); // 50K messages
    
    let start_time = web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0);
    
    let mut message_count = 0;
    while !benchmark.is_complete() {
        let payload = RSocketFactory::payload(
            format!("Benchmark message #{}", message_count).into(),
            format!("meta-{}", message_count).into()
        );
        
        let _response = client.request_response(payload).await?;
        benchmark.record_message();
        message_count += 1;
        
        if message_count % 1000 == 0 {
            console::log_1(&format!("  Processed {} messages", message_count).into());
        }
    }
    
    Ok(benchmark.get_results())
}

async fn compare_transfer_methods() -> Result<()> {
    console::log_1(&"Testing with SharedArrayBuffer...".into());
    let shared_config = WebWorkersConfig::default()
        .with_shared_buffer_size(1024 * 1024)
        .with_performance_monitoring(true);
    
    let shared_result = benchmark_transfer_method(shared_config).await?;
    
    console::log_1(&"Testing with regular transfer...".into());
    let regular_config = WebWorkersConfig::default()
        .with_shared_buffer_size(0) // Disable shared buffer
        .with_performance_monitoring(true);
    
    let regular_result = benchmark_transfer_method(regular_config).await?;
    
    let improvement = (shared_result.throughput_messages_per_sec / regular_result.throughput_messages_per_sec - 1.0) * 100.0;
    
    console::log_1(&format!("SharedArrayBuffer throughput: {:.0} msg/sec", shared_result.throughput_messages_per_sec).into());
    console::log_1(&format!("Regular transfer throughput: {:.0} msg/sec", regular_result.throughput_messages_per_sec).into());
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

async fn benchmark_transfer_method(config: WebWorkersConfig) -> Result<rsocket_rust_transport_wasm::webworkers::BenchmarkResults> {
    let transport = WebWorkersClientTransport::new("ws://localhost:7878", config)?;
    
    let client = RSocketFactory::connect()
        .transport(transport)
        .start()
        .await?;
    
    let mut benchmark = create_performance_benchmark()
        .with_duration(3000.0) // 3 seconds
        .with_message_count(10000); // 10K messages
    
    while !benchmark.is_complete() {
        let payload = RSocketFactory::payload(
            "Transfer method test".into(),
            "metadata".into()
        );
        
        let _response = client.request_response(payload).await?;
        benchmark.record_message();
    }
    
    Ok(benchmark.get_results())
}
