# RSocket Iroh P2P WASM Transport

A high-performance RSocket transport implementation for WASM environments using Iroh P2P networking with WebWorkers enhancement.

## Features

- **Browser P2P Networking**: Direct peer-to-peer connections using WebRTC
- **WebWorkers Performance**: Enhanced throughput with parallel frame processing
- **Zero-Copy Optimization**: SharedArrayBuffer support for maximum performance
- **Adaptive Configuration**: Automatic capability detection and optimization
- **Comprehensive Monitoring**: Real-time performance metrics and connection stats

## Performance Targets

- **P2P Mode**: 500K-800K messages/sec with <1ms latency
- **WebWorkers Enhancement**: 2-4x throughput improvement
- **Browser Compatibility**: Chrome, Firefox, Edge with WebRTC support

## Quick Start

### Basic Usage

```rust
use rsocket_rust_transport_iroh_wasm::IrohWasmClientTransport;

// Create basic P2P transport
let transport = IrohWasmClientTransport::from("wss://signaling.example.com/iroh-p2p");
let connection = transport.connect().await?;
```

### WebWorkers Enhanced Usage

```rust
use rsocket_rust_transport_iroh_wasm::webworkers::{
    IrohWasmWebWorkersTransport,
    create_iroh_wasm_optimized_config,
};

// Create optimized WebWorkers transport
let config = create_iroh_wasm_optimized_config();
let transport = IrohWasmWebWorkersTransport::new(
    "wss://signaling.example.com/iroh-p2p".to_string(),
    config
);

let mut connection = transport.connect().await?;

// Process frames with WebWorkers enhancement
let frame = vec![0u8; 1024];
connection.process_p2p_frame_with_workers(frame).await?;
```

## Architecture

### Core Components

- **IrohWasmClientTransport**: Basic P2P transport using WebRTC
- **IrohWasmWebWorkersTransport**: Enhanced transport with WebWorkers
- **IrohWasmDataChannel**: WebRTC data channel abstraction
- **IrohWasmWorkerPool**: Dynamic worker management and load balancing

### WebWorkers Integration

The WebWorkers implementation provides:

- **Parallel Frame Processing**: Distribute frame processing across multiple workers
- **Zero-Copy Transfers**: SharedArrayBuffer optimization when available
- **Dynamic Scaling**: Adaptive worker count based on browser capabilities
- **Performance Monitoring**: Real-time metrics and optimization feedback

## Configuration

### IrohWasmConfig

```rust
IrohWasmConfig {
    ice_servers: vec!["stun:stun.l.google.com:19302".to_string()],
    enable_webworkers: true,
    worker_count: 4,
    buffer_size: 1024 * 1024, // 1MB
    enable_performance_monitoring: true,
    connection_timeout_ms: 30000,
    max_retries: 3,
}
```

### WebWorkers Configuration

```rust
IrohWasmWebWorkersConfig {
    iroh_config: IrohWasmConfig::default(),
    webworkers_config: WebWorkersConfig::default(),
    enable_p2p_optimization: true,
    enable_webrtc_fallback: true,
    max_peer_connections: 10,
}
```

## Examples

### P2P Demo

```bash
# Build the P2P demo
cargo build --example iroh-wasm-p2p-demo --target wasm32-unknown-unknown --features webworkers

# Run in browser
wasm-pack build --target web --out-dir pkg
```

### Echo Client

```bash
# Build the echo client
cargo build --example iroh-wasm-echo-client --target wasm32-unknown-unknown --features webworkers
```

### Performance Testing

```bash
# Build the performance test
cargo build --example iroh-wasm-webworkers-test --target wasm32-unknown-unknown --features webworkers
```

## Browser Compatibility

| Browser | WebRTC | WebWorkers | SharedArrayBuffer | Status |
|---------|--------|------------|-------------------|--------|
| Chrome 97+ | ✅ | ✅ | ✅ | Full Support |
| Firefox 114+ | ✅ | ✅ | ✅ | Full Support |
| Edge 97+ | ✅ | ✅ | ✅ | Full Support |
| Safari 15+ | ✅ | ✅ | ⚠️ | Limited Support |

## Performance Optimization

### Capability Detection

The transport automatically detects browser capabilities:

```rust
let capabilities = IrohWasmWebWorkersTransport::get_capabilities();
println!("WebRTC: {}", capabilities.webrtc_supported);
println!("WebWorkers: {}", capabilities.webworkers_supported);
println!("SharedArrayBuffer: {}", capabilities.shared_array_buffer_supported);
```

### Optimized Configuration

```rust
// Create configuration optimized for detected capabilities
let config = create_iroh_wasm_optimized_config();

// Manual optimization
let mut config = IrohWasmWebWorkersConfig::default();
config.webworkers_config.worker_count = 8; // High-end devices
config.webworkers_config.buffer_size = 2 * 1024 * 1024; // 2MB
config.webworkers_config.enable_zero_copy = true;
```

## Monitoring and Debugging

### Performance Metrics

```rust
// Get real-time performance metrics
let metrics = connection.get_performance_metrics();
println!("Throughput: {:.0} msg/sec", metrics.throughput_messages_per_sec);
println!("Latency: {:.2} ms", metrics.average_latency_ms);

// Log performance summary
connection.log_performance_summary();
```

### Connection Stats

```rust
let stats = connection.get_p2p_connection_stats();
println!("Connection State: {}", stats.connection_state);
println!("Is Connected: {}", stats.is_connected);
```

## Development

### Building

```bash
# Build for WASM target
cargo build --target wasm32-unknown-unknown --features webworkers

# Build with all features
cargo build --target wasm32-unknown-unknown --features "webworkers,wasm-only"

# Run tests
cargo test --features webworkers
```

### Features

- `webworkers`: Enable WebWorkers support
- `wasm-only`: WASM-only compilation mode

## Integration with RSocket

This transport integrates seamlessly with the RSocket ecosystem:

```rust
use rsocket_rust::RSocketFactory;
use rsocket_rust_transport_iroh_wasm::webworkers::IrohWasmWebWorkersTransport;

// Create RSocket client with Iroh P2P transport
let transport = IrohWasmWebWorkersTransport::from("wss://signaling.example.com");
let client = RSocketFactory::connect()
    .transport(transport)
    .start()
    .await?;

// Use standard RSocket patterns
let response = client
    .request_response(payload!("Hello", "World"))
    .await?;
```

## Contributing

This implementation follows the comprehensive unit-by-unit development plan for RSocket Rust. See the main repository for contribution guidelines.

## License

Apache License 2.0 - see the main repository for details.
