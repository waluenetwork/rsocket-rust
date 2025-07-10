use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};
use rsocket_rust::transport::{ServerTransport, Transport};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🚀 Testing Basic Iroh P2P Connection...");
    
    println!("📡 Starting Iroh P2P server...");
    let mut server_transport = IrohServerTransport::default();
    server_transport.start().await?;
    
    println!("⏳ Waiting for server to establish NodeAddr...");
    tokio::time::sleep(Duration::from_millis(5000)).await;
    
    let server_node_addr = if let Some(node_addr) = server_transport.node_addr().await {
        println!("🆔 Server NodeAddr: {:?}", node_addr);
        node_addr
    } else {
        return Err("Failed to get server NodeAddr".into());
    };
    
    println!("🔌 Testing client connection to server...");
    let client_transport = IrohClientTransport::from_node_addr(server_node_addr);
    
    match client_transport.connect().await {
        Ok(connection) => {
            println!("✅ SUCCESS: Iroh P2P connection established!");
            println!("🔗 Connection: {:?}", connection);
        }
        Err(e) => {
            println!("❌ FAILED: Could not establish Iroh P2P connection: {:?}", e);
            return Err(e.into());
        }
    }
    
    println!("🎉 Basic Iroh P2P Connection Test PASSED!");
    Ok(())
}
