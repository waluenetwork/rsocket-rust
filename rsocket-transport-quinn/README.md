# RSocket Quinn QUIC Transport

A Quinn QUIC transport implementation for RSocket Rust.

## Features

- **QUIC Protocol**: Built on top of the Quinn QUIC implementation
- **Multiplexing**: Multiple RSocket connections can share a single QUIC connection
- **Low Latency**: Benefits from QUIC's 0-RTT connection establishment
- **Connection Migration**: QUIC's built-in connection migration support
- **TLS Security**: Built-in TLS 1.3 encryption

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rsocket_rust = "0.7"
rsocket_rust_transport_quinn = "0.7"
```

### Server Example

```rust
use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_quinn::QuinnServerTransport;

#[tokio::main]
async fn main() -> rsocket_rust::Result<()> {
    RSocketFactory::receive()
        .transport(QuinnServerTransport::from("127.0.0.1:7878"))
        .acceptor(Box::new(|setup, _socket| {
            println!("New QUIC connection: {:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .serve()
        .await
}
```

### Client Example

```rust
use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_quinn::QuinnClientTransport;

#[tokio::main]
async fn main() -> rsocket_rust::Result<()> {
    let client = RSocketFactory::connect()
        .transport(QuinnClientTransport::from("127.0.0.1:7878"))
        .acceptor(Box::new(|| Box::new(EchoRSocket)))
        .start()
        .await?;

    let req = Payload::builder()
        .set_data_utf8("Hello QUIC!")
        .build();
    
    let res = client.request_response(req).await?;
    println!("Response: {:?}", res);
    
    Ok(())
}
```

## Configuration

The transport uses self-signed certificates for development. For production use, you should configure proper TLS certificates.

## WebTransport Support

This package includes WebTransport support for native server applications:

```rust
use rsocket_rust::prelude::*;
use rsocket_rust_transport_quinn::webtransport::{WebTransportClientTransport, WebTransportServerTransport};

// WebTransport server (native only)
let server_transport = WebTransportServerTransport::new("0.0.0.0:4433".parse()?);
let server = RSocketFactory::receive()
    .transport(server_transport)
    .start()
    .await?;

// WebTransport client (native only)  
let client_transport = WebTransportClientTransport::new("https://localhost:4433".to_string());
let client = RSocketFactory::connect()
    .transport(client_transport)
    .start()
    .await?;
```

**Note**: WebTransport support is for native targets only. For browser-based WebTransport clients, use the `rsocket-transport-wasm` package.

## QUIC Benefits

- **Multiplexing**: Multiple streams over a single connection
- **0-RTT**: Faster connection establishment for repeat connections
- **Connection Migration**: Seamless network changes (WiFi to cellular)
- **Improved Congestion Control**: Modern algorithms for better performance
- **Built-in Security**: TLS 1.3 encryption by default

## License

Licensed under the Apache License, Version 2.0.
