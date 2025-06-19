use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_quinn::{QuinnClientTransport, QuinnServerTransport};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🚀 Starting Quinn QUIC Transport Integration Test...");
    
    let addr: SocketAddr = "127.0.0.1:7878".parse().unwrap();
    
    println!("📡 Starting Quinn QUIC server...");
    
    let server_task = tokio::spawn(async move {
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
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("🔌 Connecting Quinn QUIC client...");
    
    let client_result = timeout(Duration::from_secs(5), async {
        RSocketFactory::connect()
            .transport(QuinnClientTransport::from("127.0.0.1:7878"))
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
            
            let response_result = timeout(Duration::from_secs(5), client.request_response(req)).await;
            
            match response_result {
                Ok(Ok(response)) => {
                    println!("📥 Response received: {:?}", response);
                    println!("🎉 Quinn QUIC Transport Integration Test PASSED!");
                    println!("   ✅ Server started successfully");
                    println!("   ✅ Client connected over QUIC");
                    println!("   ✅ Request/response communication works");
                    println!("   ✅ RSocket over QUIC is functional");
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
