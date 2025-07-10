use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_quinn::QuinnServerTransport;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let addr: SocketAddr = "127.0.0.1:7878".parse().unwrap();
    println!("Starting Quinn QUIC echo server on {}", addr);
    
    RSocketFactory::receive()
        .transport(QuinnServerTransport::from(addr))
        .acceptor(Box::new(|setup, _socket| {
            println!("New QUIC connection established: setup={:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .on_start(Box::new(|| println!("Quinn QUIC echo server started successfully!")))
        .serve()
        .await
}
