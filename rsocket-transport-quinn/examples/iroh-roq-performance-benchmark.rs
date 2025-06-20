
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use rsocket_rust::prelude::*;
use rsocket_rust_transport_quinn::iroh_roq::{IrohRoqClientTransport, IrohRoqSessionConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let server_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let _config = IrohRoqSessionConfig::default();
    
    println!("🏃 Starting iroh-roq RTP performance benchmark");
    println!("🎯 Target: 800K+ messages/sec, <0.5ms latency");
    
    let transport = IrohRoqClientTransport::with_defaults(server_addr).await?;
    let client = RSocketFactory::connect()
        .transport(transport)
        .start()
        .await?;
    
    println!("🔥 Warming up...");
    for _ in 0..1000 {
        let _response = client
            .request_response(Payload::from("warmup"))
            .await?;
    }
    
    let message_count = 100_000;
    let payload = Payload::from("benchmark_message");
    
    println!("📊 Running benchmark: {} messages", message_count);
    let start = Instant::now();
    
    for i in 0..message_count {
        let _response = client
            .request_response(payload.clone())
            .await?;
        
        if i % 10_000 == 0 {
            let elapsed = start.elapsed();
            let rate = (i + 1) as f64 / elapsed.as_secs_f64();
            println!("  Progress: {} messages, {:.0} msg/sec", i + 1, rate);
        }
    }
    
    let total_time = start.elapsed();
    let throughput = message_count as f64 / total_time.as_secs_f64();
    let avg_latency = total_time / message_count;
    
    println!("✅ Benchmark Results:");
    println!("  Messages: {}", message_count);
    println!("  Total time: {:.2?}", total_time);
    println!("  Throughput: {:.0} messages/sec", throughput);
    println!("  Average latency: {:.2?}", avg_latency);
    
    if throughput >= 800_000.0 {
        println!("🎉 SUCCESS: Achieved target throughput!");
    } else {
        println!("⚠️  Below target throughput of 800K msg/sec");
    }
    
    if avg_latency <= Duration::from_micros(500) {
        println!("🎉 SUCCESS: Achieved target latency!");
    } else {
        println!("⚠️  Above target latency of 0.5ms");
    }
    
    Ok(())
}
