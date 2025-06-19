use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_quinn::{QuinnClientTransport, QuinnServerTransport};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🚀 Starting Quinn QUIC All RSocket Patterns Test...");
    
    let addr: SocketAddr = "127.0.0.1:7880".parse().unwrap();
    
    println!("📡 Starting Quinn QUIC server...");
    let mut server_transport = QuinnServerTransport::from(addr);
    server_transport.start().await?;
    
    let server_task = tokio::spawn(async move {
        let server_socket = RSocketFactory::receive()
            .transport(server_transport)
            .acceptor(Box::new(|setup, _socket| {
                println!("✅ Server: Setup received: {:?}", setup);
                Ok(Box::new(EchoRSocket))
            }))
            .serve();
        
        if let Err(e) = server_socket.await {
            eprintln!("❌ Server socket error: {:?}", e);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    test_request_response().await?;
    test_fire_and_forget().await?;
    test_request_stream().await?;
    test_request_channel().await?;
    
    println!("🎉 All RSocket Patterns Test PASSED!");
    println!("   ✅ Request-Response works over QUIC");
    println!("   ✅ Fire-and-Forget works over QUIC");
    println!("   ✅ Request-Stream works over QUIC");
    println!("   ✅ Request-Channel works over QUIC");
    
    server_task.abort();
    Ok(())
}

async fn test_request_response() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Testing Request-Response Pattern...");
    
    let client_transport = QuinnClientTransport::from("127.0.0.1:7880");
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let req = Payload::builder()
        .set_data_utf8("Request-Response Test")
        .build();
    
    let response = timeout(Duration::from_secs(5), client.request_response(req)).await??;
    
    match response {
        Some(payload) => {
            println!("✅ Request-Response: Received response: {:?}", 
                     String::from_utf8_lossy(payload.data().map_or(&[], |v| v)));
        }
        None => {
            return Err("No response received".into());
        }
    }
    
    Ok(())
}

async fn test_fire_and_forget() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 Testing Fire-and-Forget Pattern...");
    
    let client_transport = QuinnClientTransport::from("127.0.0.1:7880");
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let req = Payload::builder()
        .set_data_utf8("Fire-and-Forget Test")
        .build();
    
    timeout(Duration::from_secs(5), client.fire_and_forget(req)).await??;
    
    println!("✅ Fire-and-Forget: Message sent successfully (no response expected)");
    
    Ok(())
}

async fn test_request_stream() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📡 Testing Request-Stream Pattern...");
    
    let client_transport = QuinnClientTransport::from("127.0.0.1:7880");
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let req = Payload::builder()
        .set_data_utf8("Request-Stream Test")
        .build();
    
    let mut stream = client.request_stream(req);
    
    let mut count = 0;
    while let Some(result) = timeout(Duration::from_secs(5), stream.next()).await? {
        match result {
            Ok(payload) => {
                count += 1;
                println!("✅ Request-Stream: Received item {}: {:?}", 
                         count, String::from_utf8_lossy(payload.data().map_or(&[], |v| v)));
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
    
    println!("✅ Request-Stream: Received {} items from stream", count);
    
    Ok(())
}

async fn test_request_channel() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Testing Request-Channel Pattern...");
    
    let client_transport = QuinnClientTransport::from("127.0.0.1:7880");
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let input_payloads = vec![
        Ok(Payload::builder().set_data_utf8("Channel Message 1").build()),
        Ok(Payload::builder().set_data_utf8("Channel Message 2").build()),
        Ok(Payload::builder().set_data_utf8("Channel Message 3").build()),
    ];
    
    let input_stream = Box::pin(futures::stream::iter(input_payloads));
    let mut response_stream = client.request_channel(input_stream);
    
    let mut count = 0;
    while let Some(result) = timeout(Duration::from_secs(5), response_stream.next()).await? {
        match result {
            Ok(payload) => {
                count += 1;
                println!("✅ Request-Channel: Received response {}: {:?}", 
                         count, String::from_utf8_lossy(payload.data().map_or(&[], |v| v)));
                
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
    
    println!("✅ Request-Channel: Received {} responses from channel", count);
    
    Ok(())
}
