
use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_quinn::iroh_roq::{IrohRoqServerTransport, IrohRoqSessionConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let addr: std::net::SocketAddrV4 = "127.0.0.1:8080".parse()?;
    let config = IrohRoqSessionConfig {
        max_flows: 1000,
        recv_buffer_size: 64,
        enable_datagrams: true,
        enable_streams: true,
    };
    
    let mut server_transport = IrohRoqServerTransport::new(addr, config);
    
    println!("🚀 Starting iroh-roq RTP over QUIC echo server on {}", addr);
    println!("📊 Configuration:");
    println!("  - Max flows: 1000");
    println!("  - Datagrams: enabled (low-latency)");
    println!("  - Streams: enabled (reliability)");
    
    RSocketFactory::receive()
        .transport(server_transport)
        .acceptor(Box::new(|setup, _socket| {
            println!("📡 New RTP connection: {:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .serve()
        .await
}
