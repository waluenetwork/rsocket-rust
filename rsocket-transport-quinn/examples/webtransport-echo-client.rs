use rsocket_rust::prelude::*;
use rsocket_rust::Result;
use rsocket_rust_transport_quinn::webtransport::WebTransportClientTransport;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let transport = WebTransportClientTransport::from("https://127.0.0.1:4433");
    
    let client = RSocketFactory::connect()
        .transport(transport)
        .setup(Payload::from("Hello RSocket!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await?;

    let req = Payload::builder()
        .set_data_utf8("Hello WebTransport!")
        .set_metadata_utf8("metadata")
        .build();

    let res = client.request_response(req).await?;
    if let Some(payload) = res {
        println!("WebTransport Response: data={:?}", payload.data_utf8());
    }

    Ok(())
}
