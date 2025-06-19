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
    
    let server_socket = RSocketFactory::receive()
        .transport(server_transport)
        .acceptor(Box::new(|setup, _socket| {
            println!("✅ Setup received from Iroh P2P peer: {:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .serve();
    
    println!("✅ Iroh P2P server started!");
    println!("📡 Server listening for Iroh P2P connections...");
    println!("🔗 Use the server's NodeId to connect from clients");
    
    if let Err(e) = server_socket.await {
        eprintln!("❌ Server error: {:?}", e);
    }
    
    Ok(())
}
