use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    println!("🔍 Debug RSocket Setup over Iroh P2P...");
    
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
                println!("✅ Server: Setup received: {:?}", setup);
                println!("📋 Setup data: {:?}", setup.data().map(|d| String::from_utf8_lossy(d)));
                println!("📋 Setup metadata: {:?}", setup.metadata().map(|m| String::from_utf8_lossy(m)));
                Ok(Box::new(EchoRSocket))
            }))
            .serve();
        
        if let Err(e) = server_socket.await {
            eprintln!("❌ Server socket error: {:?}", e);
        }
    });
    
    println!("⏳ Waiting for Iroh P2P network discovery (5 seconds)...");
    tokio::time::sleep(Duration::from_millis(5000)).await;
    
    println!("🔌 Creating client transport...");
    let client_transport = IrohClientTransport::from_node_addr(server_node_addr);
    
    println!("🤝 Attempting RSocket connection with setup...");
    match timeout(Duration::from_secs(15), 
        RSocketFactory::connect()
            .setup(Payload::builder()
                .set_data_utf8("Debug Setup Data")
                .set_metadata_utf8("Debug Setup Metadata")
                .build())
            .transport(client_transport)
            .start()
    ).await {
        Ok(Ok(client)) => {
            println!("✅ RSocket connection established successfully!");
            
            println!("🧪 Testing simple request-response...");
            let req = Payload::builder()
                .set_data_utf8("Simple test message")
                .build();
            
            match timeout(Duration::from_secs(10), client.request_response(req)).await {
                Ok(Ok(Some(response))) => {
                    let response_data = String::from_utf8_lossy(response.data().map_or(&[], |v| v));
                    println!("✅ Request-Response successful: {}", response_data);
                }
                Ok(Ok(None)) => {
                    println!("❌ Request-Response: No response received");
                }
                Ok(Err(e)) => {
                    println!("❌ Request-Response error: {:?}", e);
                }
                Err(_) => {
                    println!("❌ Request-Response timeout");
                }
            }
        }
        Ok(Err(e)) => {
            println!("❌ Failed to establish RSocket connection: {:?}", e);
        }
        Err(_) => {
            println!("❌ RSocket connection timeout");
        }
    }
    
    server_task.abort();
    Ok(())
}
