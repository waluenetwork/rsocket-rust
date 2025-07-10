use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_quinn::{QuinnClientTransport, QuinnServerTransport};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    println!("🚀 Starting Quinn QUIC Transport Integration Test...");
    
    let addr: SocketAddr = "127.0.0.1:7878".parse().unwrap();
    
    println!("📡 Starting Quinn QUIC server...");
    
    let server_task = tokio::spawn(async move {
        println!("🔧 Starting RSocket server with Quinn transport...");
        
        let result = RSocketFactory::receive()
            .transport(QuinnServerTransport::from(addr))
            .acceptor(Box::new(|setup, _socket| {
                println!("✅ New QUIC connection established: setup={:?}", setup);
                Ok(Box::new(EchoRSocket))
            }))
            .on_start(Box::new(|| println!("🎯 Quinn QUIC server started successfully!")))
            .serve()
            .await;
        
        if let Err(e) = result {
            println!("❌ Server error: {:?}", e);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    println!("🔌 Connecting Quinn QUIC client...");
    
    let client_result = timeout(Duration::from_secs(10), async {
        println!("🔧 Creating client transport...");
        let client_transport = QuinnClientTransport::from("127.0.0.1:7878");
        
        println!("🔧 Starting RSocket client...");
        RSocketFactory::connect()
            .transport(client_transport)
            .acceptor(Box::new(|| Box::new(EchoRSocket)))
            .start()
            .await
    }).await;
    
    match client_result {
        Ok(Ok(client)) => {
            println!("✅ Client connected successfully!");
            
            println!("📤 Sending request over QUIC...");
            
            let req = Payload::builder()
                .set_data_utf8("Hello from Quinn QUIC client!")
                .build();
            
            let response_result = timeout(Duration::from_secs(10), client.request_response(req)).await;
            
            match response_result {
                Ok(Ok(Some(response))) => {
                    let data = response.data_utf8().unwrap_or("No data");
                    println!("📥 Response received: {}", data);
                    println!("🎉 Quinn QUIC Transport Integration Test PASSED!");
                    println!("   ✅ Server started successfully");
                    println!("   ✅ Client connected over QUIC");
                    println!("   ✅ Request/response communication works");
                    println!("   ✅ RSocket over QUIC is functional");
                }
                Ok(Ok(None)) => {
                    println!("❌ Request returned None response");
                    return Err("No response received".into());
                }
                Ok(Err(e)) => {
                    println!("❌ Request failed: {:?}", e);
                    return Err(e.into());
                }
                Err(_) => {
                    println!("❌ Request timed out");
                    return Err("Request timeout".into());
                }
            }
        }
        Ok(Err(e)) => {
            println!("❌ Client connection failed: {:?}", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("❌ Client connection timed out");
            return Err("Connection timeout".into());
        }
    }
    
    server_task.abort();
    
    Ok(())
}
