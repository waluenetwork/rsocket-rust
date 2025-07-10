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
    
    println!("🚀 Starting Debug Iroh P2P RSocket Handshake Test...");
    
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
        println!("🔄 Server: Starting to accept connections...");
        let server_socket = RSocketFactory::receive()
            .transport(server_transport)
            .acceptor(Box::new(|setup, _socket| {
                println!("✅ Server: RSocket Setup received! Data: {:?}", 
                        setup.data().map(|d| String::from_utf8_lossy(d)));
                println!("✅ Server: Setup metadata: {:?}", 
                        setup.metadata().map(|m| String::from_utf8_lossy(m)));
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
    
    println!("🤝 Attempting RSocket connection establishment...");
    match timeout(Duration::from_secs(10), 
                  RSocketFactory::connect()
                      .setup(Payload::builder()
                          .set_data_utf8("Debug handshake test")
                          .set_metadata_utf8("debug-metadata")
                          .build())
                      .transport(client_transport)
                      .start()).await {
        Ok(Ok(client)) => {
            println!("✅ RSocket connection established successfully!");
            
            println!("📤 Sending test request...");
            let req = Payload::builder()
                .set_data_utf8("Hello from debug test!")
                .build();
            
            match timeout(Duration::from_secs(5), client.request_response(req)).await {
                Ok(Ok(Some(response))) => {
                    let response_data = String::from_utf8_lossy(response.data().map_or(&[], |v| v));
                    println!("📨 Response received: {}", response_data);
                    println!("🎉 Debug Test PASSED!");
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
            println!("❌ Failed to establish RSocket connection: {:?}", e);
        }
        Err(_) => {
            println!("❌ RSocket connection timeout");
        }
    }
    
    server_task.abort();
    Ok(())
}
