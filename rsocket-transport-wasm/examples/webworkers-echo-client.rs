//! 

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_wasm::webworkers::{WebWorkersClientTransport, create_optimized_config, log_webworkers_info};

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    spawn_local(async {
        if let Err(e) = run_webworkers_echo_client().await {
            web_sys::console::error_1(&format!("WebWorkers echo client failed: {:?}", e).into());
        }
    });
}

async fn run_webworkers_echo_client() -> Result<()> {
    if !WebWorkersClientTransport::is_supported() {
        web_sys::console::warn_1(&"WebWorkers not supported in this environment".into());
        return Ok(());
    }

    let config = create_optimized_config();
    log_webworkers_info(&config);

    let transport = WebWorkersClientTransport::new(
        "ws://localhost:7878".to_string(),
        config
    );

    web_sys::console::log_1(&"Connecting to WebSocket server with WebWorkers enhancement...".into());

    let client = RSocketFactory::connect()
        .transport(transport)
        .acceptor(Box::new(|| {
            Box::new(EchoRSocket)
        }))
        .start()
        .await?;

    web_sys::console::log_1(&"Connected! Starting performance test...".into());

    let test_payload = Payload::builder()
        .set_data_utf8("Hello WebWorkers!")
        .set_metadata_utf8("performance-test")
        .build();

    let start_time = js_sys::Date::now();
    let mut successful_requests = 0;
    let target_requests = 1000;

    for i in 0..target_requests {
        match client.request_response(test_payload.clone()).await {
            Ok(response) => {
                successful_requests += 1;
                if i % 100 == 0 {
                    web_sys::console::log_1(&format!("Completed {} requests", i).into());
                }
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Request {} failed: {:?}", i, e).into());
            }
        }
    }

    let elapsed_ms = js_sys::Date::now() - start_time;
    let throughput = (successful_requests as f64 / elapsed_ms) * 1000.0;

    web_sys::console::log_1(&format!(
        "WebWorkers Performance Test Results:\n\
         - Successful requests: {}/{}\n\
         - Total time: {:.2}ms\n\
         - Throughput: {:.0} requests/sec\n\
         - Average latency: {:.2}ms",
        successful_requests,
        target_requests,
        elapsed_ms,
        throughput,
        elapsed_ms / successful_requests as f64
    ).into());

    Ok(())
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
