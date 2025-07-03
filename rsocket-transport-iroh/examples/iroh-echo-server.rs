use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::IrohServerTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Starting Iroh P2P Echo Server...");
    
    let mut server_transport = IrohServerTransport::default();
    server_transport.start().await?;
    
    println!("✅ Iroh P2P server started!");
    
    if let Some(node_addr_str) = server_transport.node_addr_string().await {
        println!("🔗 Complete NodeAddr for clients: {}", node_addr_str);
        println!("📋 Use this complete address for distributed connections");
        
        if let Some(node_id) = server_transport.node_id() {
            println!("💡 For remote clients, use: cargo +nightly run --example iroh-echo-client '{}'", node_id);
        }
    } else {
        println!("⚠️  Could not get complete NodeAddr - distributed connections may fail");
    }
    
    let server_socket = RSocketFactory::receive()
        .transport(server_transport)
        .acceptor(Box::new(|setup, _socket| {
            println!("✅ Setup received from Iroh P2P peer: {:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .serve();
    
    println!("📡 Server listening for Iroh P2P connections...");
    
    if let Err(e) = server_socket.await {
        eprintln!("❌ Server error: {:?}", e);
    }
    
    Ok(())
}
