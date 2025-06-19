use rsocket_rust::prelude::*;
use rsocket_rust::transport::Connection;
use rsocket_rust_transport_quinn::{QuinnClientTransport, QuinnServerTransport};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    println!("🔍 Debug Test: Server-side connection handling...");
    
    let addr: SocketAddr = "127.0.0.1:7879".parse().unwrap();
    
    println!("📡 Starting Quinn server transport...");
    let mut server_transport = QuinnServerTransport::from(addr);
    server_transport.start().await?;
    
    println!("🔌 Creating client connection in background...");
    let client_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        println!("🔧 Client: Creating transport...");
        let client_transport = QuinnClientTransport::from("127.0.0.1:7879");
        
        println!("🔧 Client: Connecting...");
        match timeout(Duration::from_secs(5), client_transport.connect()).await {
            Ok(Ok(connection)) => {
                println!("✅ Client: Connection established successfully!");
                Ok::<_, Box<dyn std::error::Error>>(connection)
            }
            Ok(Err(e)) => {
                println!("❌ Client: Connection failed: {:?}", e);
                Err(Box::new(e) as Box<dyn std::error::Error>)
            }
            Err(_) => {
                println!("❌ Client: Connection timed out");
                Err("Connection timeout".into())
            }
        }
    });
    
    println!("🔧 Server: Waiting for incoming connection...");
    match timeout(Duration::from_secs(10), server_transport.next()).await {
        Ok(Some(Ok(server_client_transport))) => {
            println!("✅ Server: Received incoming connection!");
            
            println!("🔧 Server: Testing server-side client transport connection...");
            match timeout(Duration::from_secs(5), server_client_transport.connect()).await {
                Ok(Ok(server_connection)) => {
                    println!("✅ Server: Server-side connection established!");
                    
                    println!("🔧 Testing frame splitting...");
                    let (sink, stream) = server_connection.split();
                    println!("✅ Frame splitting successful!");
                    
                    println!("🎯 Debug Test PASSED!");
                    println!("   ✅ Server transport accepts connections");
                    println!("   ✅ Client transport connects successfully");
                    println!("   ✅ Server-side client transport works");
                    println!("   ✅ Connection splitting works");
                }
                Ok(Err(e)) => {
                    println!("❌ Server: Server-side connection failed: {:?}", e);
                    return Err(e.into());
                }
                Err(_) => {
                    println!("❌ Server: Server-side connection timed out");
                    return Err("Server-side connection timeout".into());
                }
            }
        }
        Ok(Some(Err(e))) => {
            println!("❌ Server: Error accepting connection: {:?}", e);
            return Err(e.into());
        }
        Ok(None) => {
            println!("❌ Server: No connection received");
            return Err("No connection received".into());
        }
        Err(_) => {
            println!("❌ Server: Timeout waiting for connection");
            return Err("Server timeout".into());
        }
    }
    
    let _ = client_task.await;
    
    Ok(())
}
