use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_quinn::webtransport::WebTransportServerTransport;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let transport = WebTransportServerTransport::from("127.0.0.1:4433");
    
    let responder = Box::new(|_setup, _socket| {
        Ok(Box::new(EchoRSocket) as Box<dyn RSocket>)
    });

    RSocketFactory::receive()
        .transport(transport)
        .acceptor(responder)
        .serve()
        .await?;

    println!("WebTransport server listening on 127.0.0.1:4433");
    println!("Browser clients can connect using: new WebTransport('https://127.0.0.1:4433')");
    
    Ok(())
}
