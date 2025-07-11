use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpServerTransport;
use rsocket_rust_transport_websocket::WebsocketServerTransport;
use rsocket_rust_transport_quinn::QuinnServerTransport;
use rsocket_rust_transport_iroh::IrohServerTransport;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 Starting Multi-Transport RSocket Echo Server");
    println!("📡 Supporting: TCP, WebSocket, QUIC (Quinn), and Iroh P2P");

    let tcp_addr: SocketAddr = "0.0.0.0:7878".parse().unwrap();
    let ws_addr: SocketAddr = "0.0.0.0:7879".parse().unwrap();
    let quic_addr: SocketAddr = "0.0.0.0:7880".parse().unwrap();

    println!("🔧 Configuring transports:");
    println!("  📞 TCP:       {}", tcp_addr);
    println!("  🌐 WebSocket: {}", ws_addr);
    println!("  ⚡ QUIC:      {}", quic_addr);
    println!("  🔗 Iroh P2P:  (dynamic NodeAddr)");

    let server = RSocketFactory::receive_multi_transport()
        .add_transport("TCP".to_string(), TcpServerTransport::from(tcp_addr))
        .add_transport("WebSocket".to_string(), WebsocketServerTransport::from(ws_addr))
        .add_transport("QUIC".to_string(), QuinnServerTransport::from(quic_addr))
        .add_transport("Iroh-P2P".to_string(), IrohServerTransport::default())
        .acceptor(Box::new(|setup, _socket| {
            println!("✅ New connection established: setup={:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .on_start(Box::new(|| {
            println!("🎉 Multi-Transport Echo Server Started!");
            println!("📋 Ready to accept connections on all transport channels");
            println!("🔄 Use Ctrl+C to stop the server");
        }));

    server.serve().await
}
