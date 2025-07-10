# RSocket Rust Iroh P2P Transport

This package provides an Iroh P2P transport implementation for RSocket Rust, enabling peer-to-peer RSocket communication with hole-punching and relay fallback capabilities.

## Features

- **Peer-to-Peer Connections**: Direct P2P connections between nodes using Iroh networking
- **Hole-punching**: Automatic NAT traversal without port forwarding
- **Relay Fallback**: Automatic fallback to relay servers when direct connection fails
- **Connection Migration**: QUIC's built-in connection migration for mobile scenarios
- **Authenticated Connections**: Cryptographic peer identity verification using NodeId

## Usage

### Server

```rust
use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_iroh::IrohServerTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_transport = IrohServerTransport::default();
    
    let server_socket = RSocketFactory::receive()
        .transport(server_transport)
        .acceptor(Box::new(|setup, _socket| {
            Ok(Box::new(EchoRSocket))
        }))
        .serve();
    
    server_socket.await?;
    Ok(())
}
```

### Client

```rust
use rsocket_rust::prelude::*;
use rsocket_rust_transport_iroh::IrohClientTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_node_id = "k51qzi5uqu5dgutdk6teql3471rsrfvq5x8ycqcgqgdvs8qx8a8hqhqnou38m7";
    
    let client_transport = IrohClientTransport::from(server_node_id);
    let client = RSocketFactory::connect()
        .transport(client_transport)
        .start()
        .await?;
    
    let req = Payload::builder()
        .set_data_utf8("Hello P2P!")
        .build();
    
    let response = client.request_response(req).await?;
    println!("Response: {:?}", response);
    
    Ok(())
}
```

## Addressing

The Iroh transport supports multiple addressing formats:

- **NodeId only**: `k51qzi5uqu5dgutdk6teql3471rsrfvq5x8ycqcgqgdvs8qx8a8hqhqnou38m7`
- **NodeId with relay**: `k51qzi5uqu5dgutdk6teql3471rsrfvq5x8ycqcgqgdvs8qx8a8hqhqnou38m7@https://relay.example.com`
- **Full NodeAddr**: Complete NodeAddr string representation

## Examples

Run the echo server:
```bash
cargo run --example iroh-echo-server
```

Run the echo client (in another terminal):
```bash
cargo run --example iroh-echo-client <server_node_id>
```

## Dependencies

- `iroh`: Peer-to-peer networking library
- `tokio`: Async runtime
- `futures`: Async utilities
- `rsocket_rust`: Core RSocket implementation
