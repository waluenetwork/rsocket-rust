use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🚀 Starting Simple Iroh P2P Connection Test...");
    
    println!("📡 Starting Iroh P2P server...");
    let mut server_transport = IrohServerTransport::default();
    server_transport.start().await?;
    
    let server_node_addr = if let Some(node_addr) = server_transport.node_addr().await {
        println!("🆔 Server NodeAddr: {:?}", node_addr);
        node_addr
    } else {
        return Err("Failed to get server NodeAddr".into());
    };
    
    let server_task = tokio::spawn(async move {
        let server_socket = RSocketFactory::receive()
            .transport(server_transport)
            .acceptor(Box::new(|setup, _socket| {
                println!("✅ Server: Setup received from Iroh P2P peer: {:?}", setup);
                Ok(Box::new(EchoRSocket))
            }))
            .serve();
        
        if let Err(e) = server_socket.await {
            eprintln!("❌ Server socket error: {:?}", e);
        }
    });
    
    println!("⏳ Waiting for Iroh P2P network discovery (10 seconds)...");
    tokio::time::sleep(Duration::from_millis(10000)).await;
    
    println!("🔌 Attempting to connect to server...");
    let client_transport = IrohClientTransport::from_node_addr(server_node_addr);
    
    match timeout(Duration::from_secs(15), RSocketFactory::connect().transport(client_transport).start()).await {
        Ok(Ok(client)) => {
            println!("✅ Successfully connected to Iroh P2P server!");
            
            println!("📤 Sending simple request-response...");
            let req = Payload::builder()
                .set_data_utf8("Hello Iroh P2P!")
                .build();
            
            match timeout(Duration::from_secs(10), client.request_response(req)).await {
                Ok(Ok(Some(response))) => {
                    let response_data = String::from_utf8_lossy(response.data().map_or(&[], |v| v));
                    println!("📨 Response received: {}", response_data);
                    println!("🎉 Simple Iroh P2P Test PASSED!");
                }
                Ok(Ok(None)) => {
                    println!("❌ No response received");
                }
                Ok(Err(e)) => {
                    println!("❌ Request-response error: {:?}", e);
                }
                Err(_) => {
                    println!("❌ Request-response timeout");
                }
            }
        }
        Ok(Err(e)) => {
            println!("❌ Failed to connect to server: {:?}", e);
        }
        Err(_) => {
            println!("❌ Connection timeout");
        }
    }
    
    server_task.abort();
    Ok(())
}
