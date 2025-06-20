# RSocket Rust JavaScript/Node.js FFI

High-performance JavaScript/Node.js bindings for RSocket Rust implementation using NAPI-RS.

## Features

- **Complete RSocket Support**: All interaction patterns (request-response, fire-and-forget, request-stream, request-channel)
- **Multiple Transports**: TCP, WebSocket, QUIC, WebTransport, iroh-roq, WASM WebWorkers, iroh P2P
- **High Performance**: Native Rust performance with JavaScript accessibility
- **Async/Await Support**: Modern JavaScript async patterns
- **Performance Monitoring**: Built-in metrics and benchmarking
- **Cross-Platform**: Works on Node.js 10+ across all platforms

## Installation

```bash
npm install rsocket-rust
```

## Quick Start

### Client Example

```javascript
const { JsRSocketClient, JsTransportConfig, JsTransportType, JsPayload } = require('rsocket-rust');

async function main() {
    // Create transport configuration
    const config = new JsTransportConfig(
        JsTransportType.tcp(),
        "127.0.0.1:7878",
        null
    );
    
    // Create and connect client
    const client = new JsRSocketClient(config);
    await client.connect();
    
    // Create payload
    const payload = JsPayload.fromString("Hello, RSocket!", null);
    
    // Send request-response
    const response = await client.requestResponse(payload);
    if (response) {
        console.log("Response:", response.getDataUtf8());
    }
    
    await client.close();
}

main().catch(console.error);
```

### Server Example

```javascript
const { JsRSocketServer, JsTransportConfig, JsTransportType } = require('rsocket-rust');

async function main() {
    const config = new JsTransportConfig(
        JsTransportType.tcp(),
        "127.0.0.1:7878",
        null
    );
    
    const server = new JsRSocketServer(config);
    
    // Set request handler
    await server.setRequestHandler(async (payload) => {
        const data = payload.getDataUtf8();
        console.log("Received:", data);
        
        // Echo response
        return JsPayload.fromString(`Echo: ${data}`, null);
    });
    
    await server.start();
    console.log("Server started on 127.0.0.1:7878");
}

main().catch(console.error);
```

## Transport Types

### TCP Transport
```javascript
const config = new JsTransportConfig(
    JsTransportType.tcp(),
    "127.0.0.1:7878",
    null
);
```

### WebSocket Transport
```javascript
const config = new JsTransportConfig(
    JsTransportType.websocket(),
    "ws://127.0.0.1:8080/rsocket",
    null
);
```

### QUIC Transport
```javascript
const config = new JsTransportConfig(
    JsTransportType.quinnQuic(),
    "127.0.0.1:7878",
    null
);
```

### WebTransport
```javascript
const config = new JsTransportConfig(
    JsTransportType.quinnWebtransport(),
    "https://127.0.0.1:7878",
    null
);
```

## Performance Optimizations

### Enable SIMD Processing
```javascript
const config = new JsTransportConfig(
    JsTransportType.tcp(),
    "127.0.0.1:7878",
    null
);
config.enableSimdProcessing();
config.enableCrossbeamOptimizations();
config.setPerformanceMode("high");
```

### WebWorkers Support (WASM)
```javascript
const config = new JsTransportConfig(
    JsTransportType.wasmWebworkers(),
    "ws://127.0.0.1:8080",
    null
);
config.enableWebworkers();
config.setWebworkersCount(4);
```

## Factory Pattern

Use the factory for simplified client/server creation:

```javascript
const { JsRSocketFactory } = require('rsocket-rust');

// Create optimized client
const client = JsRSocketFactory.createOptimizedClient(
    "tcp",
    "127.0.0.1:7878",
    true,  // enable SIMD
    false  // disable WebWorkers
);

// Create high-performance server
const server = JsRSocketFactory.createHighPerformanceServer(
    "tcp",
    "127.0.0.1:7878",
    8  // worker count
);
```

## Performance Monitoring

```javascript
const { JsPerformanceMetrics } = require('rsocket-rust');

const metrics = new JsPerformanceMetrics();

// Record operations
metrics.recordRequest();
metrics.recordResponse(15.5); // latency in ms
metrics.recordBytesSent(1024);
metrics.recordBytesReceived(512);

// Get metrics
console.log("Throughput:", metrics.getThroughputRps(), "RPS");
console.log("Average Latency:", metrics.getAverageLatencyMs(), "ms");
console.log("Error Rate:", metrics.getErrorRate());

const summary = metrics.getSummary();
console.log("Performance Summary:", summary);
```

## RSocket Patterns

### Request-Response
```javascript
const response = await client.requestResponse(payload);
```

### Fire-and-Forget
```javascript
await client.fireAndForget(payload);
```

### Request-Stream
```javascript
const stream = await client.requestStream(payload);
for (const item of stream) {
    console.log("Stream item:", item.getDataUtf8());
}
```

### Request-Channel
```javascript
const payloads = [
    JsPayload.fromString("Item 1", null),
    JsPayload.fromString("Item 2", null),
    JsPayload.fromString("Item 3", null)
];

const responses = await client.requestChannel(payloads);
for (const response of responses) {
    console.log("Channel response:", response.getDataUtf8());
}
```

## Advanced Features

### JSON Payload Support
```javascript
const jsonData = { message: "Hello", timestamp: Date.now() };
const jsonMetadata = { type: "greeting" };

const payload = JsPayload.fromJson(jsonData, jsonMetadata);
```

### Performance Benchmarking
```javascript
const { benchmarkTransportPerformance } = require('rsocket-rust');

const results = benchmarkTransportPerformance("tcp", 10000);
console.log("Benchmark Results:", results);
```

### Transport Configuration Options
```javascript
const config = new JsTransportConfig(
    JsTransportType.tcp(),
    "127.0.0.1:7878",
    null
);

// Set buffer size
config.setBufferSize(8192);

// Enable compression
config.enableCompression(true);

// Set performance mode
config.setPerformanceMode("high");

// Get all options
const options = config.getAllOptions();
console.log("Transport Options:", options);
```

## Error Handling

```javascript
try {
    await client.connect();
    const response = await client.requestResponse(payload);
} catch (error) {
    console.error("RSocket Error:", error.message);
    
    // Check if client is still connected
    if (!await client.isConnected()) {
        console.log("Client disconnected, attempting reconnection...");
        await client.connect();
    }
}
```

## Building from Source

```bash
# Install dependencies
npm install

# Build native module
npm run build

# Run tests
npm test

# Build for all platforms
npm run build:release
```

## Supported Platforms

- **Node.js**: 10.x, 12.x, 14.x, 16.x, 18.x, 20.x+
- **Platforms**: Windows (x64, ARM64), macOS (x64, ARM64), Linux (x64, ARM64, ARM)
- **Architectures**: x86_64, aarch64, armv7

## Performance Characteristics

- **Throughput**: 500K-1M+ messages/second (depending on transport and payload size)
- **Latency**: Sub-millisecond for local connections
- **Memory**: Efficient zero-copy operations where possible
- **CPU**: Multi-threaded with work-stealing queues

## License

Apache-2.0

## Contributing

See the main [RSocket Rust repository](https://github.com/rsocket/rsocket-rust) for contribution guidelines.
