use rsocket_rust::prelude::*;
use rsocket_rust_transport_iroh::IrohClientTransport;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <server_node_id>", args[0]);
        eprintln!("Example: {} k51qzi5uqu5dgutdk6teql3471rsrfvq5x8ycqcgqgdvs8qx8a8hqhqnou38m7", args[0]);
        std::process::exit(1);
    }
    
    let server_node_addr = &args[1];
    
    println!("🚀 Starting Iroh P2P Echo Client...");
    println!("🎯 Connecting to server: {}", server_node_addr);
    
    let client_transport = IrohClientTransport::from(server_node_addr.as_str());
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    println!("✅ Connected to Iroh P2P server!");
    
    let req = Payload::builder()
        .set_data_utf8("Hello from real Iroh P2P client!")
        .build();
    
    match client.request_response(req).await? {
        Some(response) => {
            println!("📨 Response: {:?}", 
                     String::from_utf8_lossy(response.data().map_or(&[], |v| v)));
        }
        None => {
            println!("❌ No response received");
        }
    }
    
    Ok(())
}
