# RSocket Python FFI Bindings

Python bindings for the RSocket Rust implementation, providing high-performance reactive streaming capabilities with support for multiple transport protocols.

## Features

- **All RSocket Interaction Patterns**: Request-Response, Fire-and-Forget, Request-Stream, Request-Channel
- **Multiple Transport Types**: TCP, WebSocket, QUIC (Quinn), Iroh P2P
- **Enhanced P1U07 Transports**: WebTransport, iroh-roq, WebWorkers (with advanced-transports feature)
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

# With advanced transports (WebTransport, iroh-roq, WebWorkers)
maturin develop --features advanced-transports
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

### Enhanced Transport Example (P1U07)

```python
import asyncio
import rsocket_rust

async def main():
    # WebTransport client (requires advanced-transports feature)
    if hasattr(rsocket_rust, 'WebTransportClientTransport'):
        transport = rsocket_rust.WebTransportClientTransport("https://localhost:4433")
        client = await rsocket_rust.RSocketFactory.connect_webtransport(transport)
        
        payload = rsocket_rust.Payload.builder().set_data_utf8("WebTransport message").build()
        response = await client.request_response(payload)
        print(f"WebTransport Response: {response.data_utf8()}")
    
    # iroh-roq real-time streaming
    if hasattr(rsocket_rust, 'IrohRoqClientTransport'):
        transport = rsocket_rust.IrohRoqClientTransport("iroh://endpoint")
        client = await rsocket_rust.RSocketFactory.connect_iroh_roq(transport)
        
        payload = rsocket_rust.Payload.builder().set_data_utf8("Real-time message").build()
        response = await client.request_response(payload)
        print(f"iroh-roq Response: {response.data_utf8()}")

asyncio.run(main())
```

## API Reference

### Core Classes

- `Payload`: Data container with optional metadata
- `PayloadBuilder`: Builder for creating payloads
- `Client`: RSocket client for all interaction patterns
- `MultiTransportServerBuilder`: Builder for multi-transport servers

### Transport Classes

#### Standard Transports
- `TcpClientTransport` / `TcpServerTransport`: TCP transport
- `WebSocketClientTransport` / `WebSocketServerTransport`: WebSocket transport  
- `QuinnClientTransport` / `QuinnServerTransport`: QUIC transport
- `IrohClientTransport` / `IrohServerTransport`: Iroh P2P transport

#### Enhanced Transports (P1U07 - requires advanced-transports feature)
- `WebTransportClientTransport` / `WebTransportServerTransport`: WebTransport for browsers
- `IrohRoqClientTransport` / `IrohRoqServerTransport`: iroh-roq real-time streaming
- `WebWorkersClientTransport`: Advanced WASM WebWorkers with SIMD optimizations
- `WebWorkersConfig`: Configuration for WebWorkers performance tuning

### Factory

- `RSocketFactory`: Factory for creating clients and servers

## Development

```bash
# Setup development environment
python -m venv venv
source venv/bin/activate
pip install maturin

# Build and install in development mode
maturin develop

# Build with enhanced transports
maturin develop --features advanced-transports

# Run tests
python examples/test_all_patterns.py
```

## License

Apache-2.0
