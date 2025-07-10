use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};
use std::time::Duration;
use tokio::time::timeout;
use futures::StreamExt;
use std::sync::OnceLock;

static SERVER_NODE_ADDR: OnceLock<iroh::NodeAddr> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🚀 Starting Comprehensive Real Iroh P2P RSocket Patterns Test...");
    
    println!("📡 Starting Real Iroh P2P server...");
    let mut server_transport = IrohServerTransport::default();
    server_transport.start().await?;
    
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
                println!("✅ Server: Setup received from Real Iroh P2P peer: {:?}", setup);
                Ok(Box::new(EchoRSocket))
            }))
            .serve();
        
        if let Err(e) = server_socket.await {
            eprintln!("❌ Server socket error: {:?}", e);
        }
    });
    
    println!("⏳ Waiting for Iroh P2P network discovery and relay establishment...");
    tokio::time::sleep(Duration::from_millis(30000)).await;
    
    test_request_response_p2p().await?;
    test_fire_and_forget_p2p().await?;
    test_request_stream_p2p().await?;
    test_request_channel_p2p().await?;
    
    println!("🎉 All RSocket Patterns Test PASSED over Real Iroh P2P!");
    println!("   ✅ Request-Response works over Real Iroh P2P");
    println!("   ✅ Fire-and-Forget works over Real Iroh P2P");
    println!("   ✅ Request-Stream works over Real Iroh P2P");
    println!("   ✅ Request-Channel works over Real Iroh P2P");
    
    server_task.abort();
    Ok(())
}

async fn test_request_response_p2p() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Testing Request-Response Pattern over Iroh P2P...");
    
    let node_addr = SERVER_NODE_ADDR.get().ok_or("Server NodeAddr not available")?.clone();
    
    println!("🔌 Connecting to server NodeAddr: {:?}", node_addr);
    let client_transport = IrohClientTransport::from_node_addr(node_addr);
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let req = Payload::builder()
        .set_data_utf8("Request-Response test over Iroh P2P")
        .build();
    
    let response = timeout(Duration::from_secs(30), client.request_response(req)).await??;
    
    match response {
        Some(payload) => {
            let response_data = String::from_utf8_lossy(payload.data().map_or(&[], |v| v));
            println!("📨 Response: {}", response_data);
            assert_eq!(response_data, "Request-Response test over Iroh P2P");
            println!("✅ Request-Response Pattern Test PASSED over Iroh P2P");
        }
        None => {
            return Err("No response received for request-response".into());
        }
    }
    
    Ok(())
}

async fn test_fire_and_forget_p2p() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 Testing Fire-and-Forget Pattern over Iroh P2P...");
    
    let node_addr = SERVER_NODE_ADDR.get().ok_or("Server NodeAddr not available")?.clone();
    
    println!("🔌 Connecting to server NodeAddr: {:?}", node_addr);
    let client_transport = IrohClientTransport::from_node_addr(node_addr);
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let req = Payload::builder()
        .set_data_utf8("Fire-and-Forget test over Iroh P2P")
        .build();
    
    timeout(Duration::from_secs(30), client.fire_and_forget(req)).await??;
    println!("✅ Fire-and-Forget Pattern Test PASSED over Iroh P2P");
    
    Ok(())
}

async fn test_request_stream_p2p() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📡 Testing Request-Stream Pattern over Iroh P2P...");
    
    let node_addr = SERVER_NODE_ADDR.get().ok_or("Server NodeAddr not available")?.clone();
    
    println!("🔌 Connecting to server NodeAddr: {:?}", node_addr);
    let client_transport = IrohClientTransport::from_node_addr(node_addr);
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let req = Payload::builder()
        .set_data_utf8("Request-Stream test over Iroh P2P")
        .build();
    
    let mut stream = client.request_stream(req);
    
    let mut count = 0;
    while let Some(result) = timeout(Duration::from_secs(30), stream.next()).await? {
        match result {
            Ok(payload) => {
                count += 1;
                let data = String::from_utf8_lossy(payload.data().map_or(&[], |v| v));
                println!("📦 Request-Stream: Received item {}: {}", count, data);
                break;
            }
            Err(e) => {
                return Err(format!("Stream error: {:?}", e).into());
            }
        }
    }
    
    if count == 0 {
        return Err("No stream items received".into());
    }
    
    println!("✅ Request-Stream Pattern Test PASSED over Iroh P2P (received {} items)", count);
    
    Ok(())
}

async fn test_request_channel_p2p() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Testing Request-Channel Pattern over Iroh P2P...");
    
    let node_addr = SERVER_NODE_ADDR.get().ok_or("Server NodeAddr not available")?.clone();
    
    println!("🔌 Connecting to server NodeAddr: {:?}", node_addr);
    let client_transport = IrohClientTransport::from_node_addr(node_addr);
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let input_payloads = vec![
        Ok(Payload::builder().set_data_utf8("Channel Message 1 over Iroh P2P").build()),
        Ok(Payload::builder().set_data_utf8("Channel Message 2 over Iroh P2P").build()),
        Ok(Payload::builder().set_data_utf8("Channel Message 3 over Iroh P2P").build()),
    ];
    
    let input_stream = Box::pin(futures::stream::iter(input_payloads));
    let mut response_stream = client.request_channel(input_stream);
    
    let mut count = 0;
    while let Some(result) = timeout(Duration::from_secs(30), response_stream.next()).await? {
        match result {
            Ok(payload) => {
                count += 1;
                let data = String::from_utf8_lossy(payload.data().map_or(&[], |v| v));
                println!("🔄 Request-Channel: Received response {}: {}", count, data);
                
                if count >= 3 {
                    break;
                }
            }
            Err(e) => {
                return Err(format!("Channel error: {:?}", e).into());
            }
        }
    }
    
    if count == 0 {
        return Err("No channel responses received".into());
    }
    
    println!("✅ Request-Channel Pattern Test PASSED over Iroh P2P (received {} responses)", count);
    
    Ok(())
}
