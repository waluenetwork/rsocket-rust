# RSocket Python FFI Bindings

Python bindings for the RSocket Rust implementation, providing high-performance reactive streaming capabilities with support for multiple transport protocols.

## Features

- **All RSocket Interaction Patterns**: Request-Response, Fire-and-Forget, Request-Stream, Request-Channel
- **Multiple Transport Types**: TCP, WebSocket, QUIC (Quinn), Iroh P2P
- **Multi-Transport Server**: Single server supporting multiple transport channels simultaneously
- **Async/Await Support**: Full Python async/await integration
- **High Performance**: Built on Rust for maximum performance

## Installation

```bash
# Install from source (requires Rust toolchain)
pip install maturin
maturin develop

# Or build wheel
maturin build --release
pip install target/wheels/rsocket_rust-*.whl
```

## Quick Start

### Client Example

```python
import asyncio
import rsocket_rust

async def main():
    # Connect via TCP
    client = await rsocket_rust.RSocketFactory.connect_tcp(
        rsocket_rust.TcpClientTransport("127.0.0.1:7878")
    )
    
    # Create payload
    payload = (rsocket_rust.Payload.builder()
               .set_data_utf8("Hello, RSocket!")
               .build())
    
    # Request-Response
    response = await client.request_response(payload)
    print(f"Response: {response.data_utf8()}")

asyncio.run(main())
```

### Multi-Transport Server Example

```python
import asyncio
import rsocket_rust

def echo_handler(setup_payload):
    print(f"New connection: {setup_payload.data_utf8()}")
    return None  # Echo responder

async def main():
    server = (rsocket_rust.MultiTransportServerBuilder()
              .add_tcp_transport("TCP", rsocket_rust.TcpServerTransport("127.0.0.1:7878"))
              .add_websocket_transport("WS", rsocket_rust.WebSocketServerTransport("127.0.0.1:7879"))
              .add_quic_transport("QUIC", rsocket_rust.QuinnServerTransport("127.0.0.1:7880"))
              .add_iroh_transport("P2P", rsocket_rust.IrohServerTransport())
              .acceptor(echo_handler))
    
    await server.serve()

asyncio.run(main())
```

## API Reference

### Core Classes

- `Payload`: Data container with optional metadata
- `PayloadBuilder`: Builder for creating payloads
- `Client`: RSocket client for all interaction patterns
- `MultiTransportServerBuilder`: Builder for multi-transport servers

### Transport Classes

- `TcpClientTransport` / `TcpServerTransport`: TCP transport
- `WebSocketClientTransport` / `WebSocketServerTransport`: WebSocket transport  
- `QuinnClientTransport` / `QuinnServerTransport`: QUIC transport
- `IrohClientTransport` / `IrohServerTransport`: Iroh P2P transport

### Factory

- `RSocketFactory`: Factory for creating clients and servers

## Examples

See the `examples/` directory for comprehensive examples demonstrating:
- All 4 RSocket interaction patterns
- All transport types
- Multi-transport server setup
- Error handling patterns

## Development

```bash
# Setup development environment
python -m venv venv
source venv/bin/activate
pip install maturin

# Build and install in development mode
maturin develop

# Run tests
python examples/test_all_patterns.py
```

## License

Apache-2.0
