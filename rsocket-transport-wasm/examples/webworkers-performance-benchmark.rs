//! 

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_wasm::{WebsocketClientTransport, webworkers::{WebWorkersClientTransport, create_optimized_config, benchmark_webworkers_performance}};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    spawn_local(async {
        if let Err(e) = run_performance_benchmark().await {
            web_sys::console::error_1(&format!("Performance benchmark failed: {:?}", e).into());
        }
    });
}

async fn run_performance_benchmark() -> Result<()> {
    web_sys::console::log_1(&"Starting WebWorkers vs Standard WebSocket Performance Benchmark".into());

    let websocket_url = "ws://localhost:7878".to_string();
    let test_duration_ms = 5000; // 5 seconds
    let target_throughput = 100_000; // 100K messages

    web_sys::console::log_1(&"Benchmarking standard WebSocket transport...".into());
    let standard_results = benchmark_standard_websocket(
        websocket_url.clone(),
        test_duration_ms,
        target_throughput
    ).await?;

    if WebWorkersClientTransport::is_supported() {
        web_sys::console::log_1(&"Benchmarking WebWorkers-enhanced transport...".into());
        let webworkers_results = benchmark_webworkers_performance(
            websocket_url,
            test_duration_ms,
            target_throughput
        ).await?;

        display_benchmark_comparison(&standard_results, &webworkers_results);
    } else {
        web_sys::console::warn_1(&"WebWorkers not supported, skipping WebWorkers benchmark".into());
        display_standard_results(&standard_results);
    }

    Ok(())
}

async fn benchmark_standard_websocket(
    websocket_url: String,
    duration_ms: u32,
    target_throughput: u32,
) -> Result<BenchmarkResults> {
    let transport = WebsocketClientTransport::from(websocket_url);
    let client = RSocketFactory::connect()
        .transport(transport)
        .acceptor(Box::new(|| Box::new(EchoRSocket)))
        .start()
        .await?;

    let test_frame = Payload::builder()
        .set_data(vec![0u8; 1024].into()) // 1KB payload
        .build();

    let start_time = js_sys::Date::now();
    let mut frames_sent = 0;

    while frames_sent < target_throughput && (js_sys::Date::now() - start_time) < duration_ms as f64 {
        match client.request_response(test_frame.clone()).await {
            Ok(_) => frames_sent += 1,
            Err(_) => break,
        }

        if frames_sent % 1000 == 0 {
            wasm_bindgen_futures::JsFuture::from(
                js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(1))
            ).await.ok();
        }
    }

    let elapsed_ms = js_sys::Date::now() - start_time;
    let messages_per_second = (frames_sent as f64 / elapsed_ms) * 1000.0;

    Ok(BenchmarkResults {
        messages_processed: frames_sent,
        elapsed_ms,
        messages_per_second,
        target_achieved: messages_per_second >= 50_000.0, // 50K msg/sec target for standard
    })
}

fn display_benchmark_comparison(
    standard: &BenchmarkResults,
    webworkers: &rsocket_rust_transport_wasm::webworkers::performance::BenchmarkResults,
) {
    let improvement_factor = webworkers.messages_per_second / standard.messages_per_second;
    let improvement_percent = (improvement_factor - 1.0) * 100.0;

    web_sys::console::log_1(&format!(
        "🚀 WebWorkers Performance Benchmark Results:\n\
         \n\
         📊 Standard WebSocket Transport:\n\
         - Messages: {}\n\
         - Duration: {:.2}ms\n\
         - Throughput: {:.0} msg/sec\n\
         - Target Achieved: {}\n\
         \n\
         ⚡ WebWorkers-Enhanced Transport:\n\
         - Messages: {}\n\
         - Duration: {:.2}ms\n\
         - Throughput: {:.0} msg/sec\n\
         - Target Achieved: {}\n\
         \n\
         🎯 Performance Improvement:\n\
         - Improvement Factor: {:.2}x\n\
         - Improvement Percentage: {:.1}%\n\
         - Performance Grade: {}",
        standard.messages_processed,
        standard.elapsed_ms,
        standard.messages_per_second,
        if standard.target_achieved { "✅" } else { "❌" },
        webworkers.messages_processed,
        webworkers.elapsed_ms,
        webworkers.messages_per_second,
        if webworkers.target_achieved { "✅" } else { "❌" },
        improvement_factor,
        improvement_percent,
        get_performance_grade(webworkers.messages_per_second)
    ).into());
}

fn display_standard_results(standard: &BenchmarkResults) {
    web_sys::console::log_1(&format!(
        "📊 Standard WebSocket Transport Results:\n\
         - Messages: {}\n\
         - Duration: {:.2}ms\n\
         - Throughput: {:.0} msg/sec\n\
         - Target Achieved: {}",
        standard.messages_processed,
        standard.elapsed_ms,
        standard.messages_per_second,
        if standard.target_achieved { "✅" } else { "❌" }
    ).into());
}

fn get_performance_grade(messages_per_second: f64) -> &'static str {
    if messages_per_second >= 800_000.0 {
        "A+ (Excellent)"
    } else if messages_per_second >= 500_000.0 {
        "A (Very Good)"
    } else if messages_per_second >= 200_000.0 {
        "B (Good)"
    } else if messages_per_second >= 100_000.0 {
        "C (Fair)"
    } else {
        "D (Needs Improvement)"
    }
}

#[derive(Debug, Clone)]
struct BenchmarkResults {
    messages_processed: u32,
    elapsed_ms: f64,
    messages_per_second: f64,
    target_achieved: bool,
}

struct EchoRSocket;

#[async_trait::async_trait]
impl RSocket for EchoRSocket {
    async fn request_response(&self, req: Payload) -> Result<Payload> {
        Ok(req)
    }

    async fn fire_and_forget(&self, _req: Payload) -> Result<()> {
        Ok(())
    }
}
