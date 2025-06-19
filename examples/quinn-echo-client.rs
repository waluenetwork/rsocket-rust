use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_quinn::QuinnClientTransport;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("Connecting to Quinn QUIC echo server...");
    
    let client = RSocketFactory::connect()
        .transport(QuinnClientTransport::from("127.0.0.1:7878"))
        .acceptor(Box::new(|| Box::new(EchoRSocket)))
        .start()
        .await
        .expect("Failed to connect to QUIC server!");

    println!("Connected! Sending request...");
    
    let req = Payload::builder()
        .set_data_utf8("Hello from Quinn QUIC client!")
        .build();
    
    let res = client
        .request_response(req)
        .await
        .expect("Request failed!");
    
    println!("Response received: {:?}", res);
    
    Ok(())
}
