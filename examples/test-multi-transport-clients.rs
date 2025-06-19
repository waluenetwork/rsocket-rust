use rsocket_rust::prelude::*;
use rsocket_rust::error::RSocketError;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use rsocket_rust_transport_quinn::QuinnClientTransport;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🧪 Testing Multi-Transport RSocket Clients");
    println!("🔗 Connecting to multi-transport server via all transport types");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let mut all_tests_passed = true;

    println!("\n📞 Testing TCP Transport...");
    match test_transport("TCP", TcpClientTransport::from("127.0.0.1:7878")).await {
        Ok(_) => println!("✅ TCP transport test passed"),
        Err(e) => {
            println!("❌ TCP transport test failed: {:?}", e);
            all_tests_passed = false;
        }
    }

    println!("\n🌐 Testing WebSocket Transport...");
    match test_transport("WebSocket", WebsocketClientTransport::from("ws://127.0.0.1:7879")).await {
        Ok(_) => println!("✅ WebSocket transport test passed"),
        Err(e) => {
            println!("❌ WebSocket transport test failed: {:?}", e);
            all_tests_passed = false;
        }
    }

    println!("\n⚡ Testing QUIC Transport...");
    match test_transport("QUIC", QuinnClientTransport::from("127.0.0.1:7880")).await {
        Ok(_) => println!("✅ QUIC transport test passed"),
        Err(e) => {
            println!("❌ QUIC transport test failed: {:?}", e);
            all_tests_passed = false;
        }
    }

    println!("\n🔗 Iroh P2P Transport:");
    println!("ℹ️  Iroh P2P test requires server NodeAddr discovery - would need separate coordination");

    if all_tests_passed {
        println!("\n🎉 All available transport tests passed!");
        println!("✅ Multi-transport server is working correctly");
    } else {
        println!("\n❌ Some transport tests failed");
    }

    Ok(())
}

async fn test_transport<T>(name: &str, transport: T) -> Result<()>
where
    T: Transport + Send + Sync + 'static,
    T::Conn: Send + Sync + 'static,
{
    let client = timeout(
        Duration::from_secs(10),
        RSocketFactory::connect()
            .transport(transport)
            .start()
    ).await??;

    let request = Payload::builder()
        .set_data_utf8(&format!("Hello from {} client!", name))
        .build();

    let response = timeout(
        Duration::from_secs(5),
        client.request_response(request)
    ).await??;

    if let Some(response) = response {
        let response_data = String::from_utf8_lossy(response.data().map_or(&[], |v| v));
        println!("📨 {} response: {}", name, response_data);
    } else {
        return Err(RSocketError::Other(anyhow::anyhow!("No response received")).into());
    }

    Ok(())
}
