use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};
use std::time::Duration;
use tokio::time::timeout;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🚀 Starting Comprehensive Real Iroh P2P RSocket Patterns Test...");
    
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
    
    println!("⏳ Waiting for Iroh P2P network discovery (8 seconds)...");
    tokio::time::sleep(Duration::from_millis(8000)).await;
    
    println!("🔌 Attempting to connect to server...");
    let client_transport = IrohClientTransport::from_node_addr(server_node_addr);
    
    match timeout(Duration::from_secs(12), RSocketFactory::connect().transport(client_transport).start()).await {
        Ok(Ok(client)) => {
            println!("✅ Successfully connected to Iroh P2P server!");
            
            let mut all_patterns_passed = true;
            
            println!("\n🧪 Testing Pattern 1: Request-Response");
            let req = Payload::builder()
                .set_data_utf8("Request-Response Test over Real Iroh P2P")
                .build();
            
            match timeout(Duration::from_secs(8), client.request_response(req)).await {
                Ok(Ok(Some(response))) => {
                    let response_data = String::from_utf8_lossy(response.data().map_or(&[], |v| v));
                    println!("✅ Request-Response: {}", response_data);
                }
                Ok(Ok(None)) => {
                    println!("❌ Request-Response: No response received");
                    all_patterns_passed = false;
                }
                Ok(Err(e)) => {
                    println!("❌ Request-Response error: {:?}", e);
                    all_patterns_passed = false;
                }
                Err(_) => {
                    println!("❌ Request-Response timeout");
                    all_patterns_passed = false;
                }
            }
            
            println!("\n🧪 Testing Pattern 2: Fire-and-Forget");
            let req = Payload::builder()
                .set_data_utf8("Fire-and-Forget Test over Real Iroh P2P")
                .build();
            
            match timeout(Duration::from_secs(5), client.fire_and_forget(req)).await {
                Ok(Ok(())) => {
                    println!("✅ Fire-and-Forget: Sent successfully");
                }
                Ok(Err(e)) => {
                    println!("❌ Fire-and-Forget error: {:?}", e);
                    all_patterns_passed = false;
                }
                Err(_) => {
                    println!("❌ Fire-and-Forget timeout");
                    all_patterns_passed = false;
                }
            }
            
            println!("\n🧪 Testing Pattern 3: Request-Stream");
            let req = Payload::builder()
                .set_data_utf8("Request-Stream Test over Real Iroh P2P")
                .build();
            
            match timeout(Duration::from_secs(10), async {
                let mut stream = client.request_stream(req);
                let mut count = 0;
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(payload) => {
                            let data = String::from_utf8_lossy(payload.data().map_or(&[], |v| v));
                            println!("📨 Request-Stream item {}: {}", count + 1, data);
                            count += 1;
                            if count >= 3 {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("❌ Request-Stream error: {:?}", e);
                            return Err(e);
                        }
                    }
                }
                Ok(count)
            }).await {
                Ok(Ok(count)) if count > 0 => {
                    println!("✅ Request-Stream: Received {} items", count);
                }
                Ok(Ok(_)) => {
                    println!("❌ Request-Stream: No items received");
                    all_patterns_passed = false;
                }
                Ok(Err(e)) => {
                    println!("❌ Request-Stream error: {:?}", e);
                    all_patterns_passed = false;
                }
                Err(_) => {
                    println!("❌ Request-Stream timeout");
                    all_patterns_passed = false;
                }
            }
            
            println!("\n🧪 Testing Pattern 4: Request-Channel");
            let req = Payload::builder()
                .set_data_utf8("Request-Channel Test over Real Iroh P2P")
                .build();
            
            match timeout(Duration::from_secs(10), async {
                let mut results = client.request_channel(Box::pin(futures::stream::iter(vec![Ok(req)])));
                
                let mut count = 0;
                while let Some(result) = results.next().await {
                    match result {
                        Ok(payload) => {
                            let data = String::from_utf8_lossy(payload.data().map_or(&[], |v| v));
                            println!("📨 Request-Channel response {}: {}", count + 1, data);
                            count += 1;
                            if count >= 1 {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("❌ Request-Channel error: {:?}", e);
                            return Err(e);
                        }
                    }
                }
                Ok(count)
            }).await {
                Ok(Ok(count)) if count > 0 => {
                    println!("✅ Request-Channel: Exchanged {} messages", count);
                }
                Ok(Ok(_)) => {
                    println!("❌ Request-Channel: No messages exchanged");
                    all_patterns_passed = false;
                }
                Ok(Err(e)) => {
                    println!("❌ Request-Channel error: {:?}", e);
                    all_patterns_passed = false;
                }
                Err(_) => {
                    println!("❌ Request-Channel timeout");
                    all_patterns_passed = false;
                }
            }
            
            if all_patterns_passed {
                println!("\n🎉 ALL RSOCKET PATTERNS PASSED OVER REAL IROH P2P!");
                println!("✅ Verified actual Iroh P2P networking with all 4 RSocket interaction patterns");
            } else {
                println!("\n❌ Some RSocket patterns failed over Real Iroh P2P");
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
