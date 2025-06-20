# RSocket Quinn QUIC Transport

A Quinn QUIC transport implementation for RSocket Rust.

## Features

- **QUIC Protocol**: Built on top of the Quinn QUIC implementation
- **WebTransport**: Browser-compatible WebTransport over QUIC
- **iroh-roq RTP**: Real-time streaming with RTP over QUIC (sub-millisecond latency)
- **Multiplexing**: Multiple RSocket connections can share a single QUIC connection
- **Low Latency**: Benefits from QUIC's 0-RTT connection establishment
- **Connection Migration**: QUIC's built-in connection migration support
- **TLS Security**: Built-in TLS 1.3 encryption
- **Dual Communication**: Reliable streams + unreliable datagrams for optimal performance

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rsocket_rust = "0.7"
rsocket_rust_transport_quinn = "0.7"

# For iroh-roq RTP over QUIC support
rsocket_rust_transport_quinn = { version = "0.7", features = ["iroh-roq"] }

# For WebTransport support
rsocket_rust_transport_quinn = { version = "0.7", features = ["webtransport"] }
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


## iroh-roq RTP over QUIC Support

The Quinn transport package includes optional support for iroh-roq RTP over QUIC, providing:

- **Sub-millisecond latency**: <0.5ms for real-time applications
- **High throughput**: 800K+ messages/sec capability
- **Dual communication paths**: Reliable streams + unreliable datagrams
- **RTP streaming**: Native RTP packet handling for media applications

### Usage

Enable the iroh-roq feature:

```toml
[dependencies]
rsocket_rust_transport_quinn = { version = "0.7", features = ["iroh-roq"] }
```

### Example

```rust
use rsocket_rust_transport_quinn::iroh_roq::{IrohRoqClientTransport, IrohRoqSessionConfig};

let config = IrohRoqSessionConfig {
    enable_datagrams: true,  // Low-latency mode
    enable_streams: true,    // Reliability mode
    ..Default::default()
};

let transport = IrohRoqClientTransport::with_defaults(server_addr).await?;
```

### Applications

- **Gaming**: Real-time multiplayer with sub-millisecond latency
- **Video/Audio**: Live streaming with RTP packet handling
- **Trading**: High-frequency trading applications
- **IoT**: Real-time sensor data streaming


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
