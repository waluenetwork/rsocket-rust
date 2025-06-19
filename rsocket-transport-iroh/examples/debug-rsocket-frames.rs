use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};
use std::time::Duration;
use tokio::time::timeout;
use std::sync::OnceLock;

static SERVER_NODE_ADDR: OnceLock<iroh::NodeAddr> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🚀 Testing RSocket Frame Exchange over Iroh P2P...");
    
    println!("📡 Starting Iroh P2P server...");
    let mut server_transport = IrohServerTransport::default();
    server_transport.start().await?;
    
    println!("⏳ Waiting for server to establish NodeAddr...");
    tokio::time::sleep(Duration::from_millis(10000)).await;
    
    let server_node_addr = if let Some(node_addr) = server_transport.node_addr().await {
        println!("🆔 Server NodeAddr: {:?}", node_addr);
        SERVER_NODE_ADDR.set(node_addr.clone()).map_err(|_| "Failed to set server NodeAddr")?;
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
    
    println!("⏳ Waiting for Iroh P2P network discovery...");
    tokio::time::sleep(Duration::from_millis(5000)).await;
    
    println!("🔌 Testing RSocket connection and frame exchange...");
    let client_transport = IrohClientTransport::from_node_addr(server_node_addr);
    
    match timeout(Duration::from_secs(15), RSocketFactory::connect()
        .transport(client_transport)
        .start()).await {
        Ok(Ok(client)) => {
            println!("✅ RSocket client connected successfully!");
            
            println!("📤 Testing request-response frame exchange...");
            let req = Payload::builder()
                .set_data_utf8("Test frame over Iroh P2P")
                .build();
            
            match timeout(Duration::from_secs(10), client.request_response(req)).await {
                Ok(Ok(Some(response))) => {
                    let response_data = String::from_utf8_lossy(response.data().map_or(&[], |v| v));
                    println!("📨 Response received: {}", response_data);
                    println!("🎉 RSocket Frame Exchange Test PASSED!");
                }
                Ok(Ok(None)) => {
                    println!("❌ No response received");
                    return Err("No response received".into());
                }
                Ok(Err(e)) => {
                    println!("❌ RSocket error: {:?}", e);
                    return Err(e.into());
                }
                Err(_) => {
                    println!("❌ Request-response timeout");
                    return Err("Request-response timeout".into());
                }
            }
        }
        Ok(Err(e)) => {
            println!("❌ RSocket connection failed: {:?}", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("❌ RSocket connection timeout");
            return Err("RSocket connection timeout".into());
        }
    }
    
    server_task.abort();
    Ok(())
}
