use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::P2PServerTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Starting Iroh P2P Echo Server...");
    
    let server_transport = P2PServerTransport::default();
    
    let server_socket = RSocketFactory::receive()
        .transport(server_transport)
        .acceptor(Box::new(|setup, _socket| {
            println!("✅ Server: Setup received from peer: {:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .serve();
    
    println!("🌐 Server ready for P2P connections!");
    println!("📋 Use this server's NodeId to connect from clients");
    
    if let Err(e) = server_socket.await {
        eprintln!("❌ Server error: {:?}", e);
    }
    
    Ok(())
}
