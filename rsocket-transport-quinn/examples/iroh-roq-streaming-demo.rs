
use std::net::SocketAddr;
use std::time::Duration;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_quinn::iroh_roq::{IrohRoqClientTransport, IrohRoqSessionConfig};
use tokio::time::interval;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let server_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let config = IrohRoqSessionConfig {
        max_flows: 100,
        recv_buffer_size: 32,
        enable_datagrams: true,
        enable_streams: true,
    };
    
    println!("🎬 Starting iroh-roq RTP streaming demo");
    println!("📡 Configuration:");
    println!("  - Server: {}", server_addr);
    println!("  - Datagrams: {} (low-latency)", config.enable_datagrams);
    println!("  - Streams: {} (reliability)", config.enable_streams);
    
    let transport = IrohRoqClientTransport::new(
        "0.0.0.0:0".parse()?,
        server_addr,
        config
    ).await?;
    
    let client = RSocketFactory::connect()
        .transport(transport)
        .start()
        .await?;
    
    println!("🔗 Connected to iroh-roq server");
    
    let mut stream_interval = interval(Duration::from_millis(16));
    let mut frame_count = 0u64;
    
    println!("🎥 Starting RTP streaming simulation (60 FPS)...");
    
    for _ in 0..300 {
        stream_interval.tick().await;
        
        let frame_data = format!("RTP_FRAME_{:06}", frame_count);
        let payload = Payload::builder().set_data_utf8(&frame_data).build();
        
        match client.request_response(payload).await {
            Ok(response) => {
                if frame_count % 60 == 0 {
                    match response {
                        Some(payload) => {
                            if let Some(data) = payload.data_utf8() {
                                println!("📺 Streamed {} frames, latest response: {}", 
                                         frame_count + 1, data);
                            } else {
                                println!("📺 Streamed {} frames, no response data", frame_count + 1);
                            }
                        }
                        None => {
                            println!("📺 Streamed {} frames, no response payload", frame_count + 1);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Frame {} failed: {:?}", frame_count, e);
            }
        }
        
        frame_count += 1;
    }
    
    println!("✅ RTP streaming demo completed!");
    println!("📊 Total frames streamed: {}", frame_count);
    println!("🎯 Average rate: 60 FPS over 5 seconds");
    
    Ok(())
}
