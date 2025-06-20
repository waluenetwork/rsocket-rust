# RSocket JavaScript/Node.js FFI Bindings

JavaScript/Node.js bindings for the RSocket Rust implementation, providing high-performance reactive streaming capabilities with native async/await support.

## Features

- **All RSocket Interaction Patterns**: Request-Response, Fire-and-Forget, Request-Stream, Request-Channel
- **Multiple Transport Types**: TCP, WebSocket, QUIC (Quinn), Iroh P2P
- **Native Async/Await**: Promise-based APIs that feel natural to JavaScript developers
- **High Performance**: Built on Rust with Neon for maximum performance
- **Node.js Integration**: Seamless integration with Node.js applications

## Installation

```bash
npm install rsocket-rust-js
```

## Quick Start

```javascript
const rsocket = require('rsocket-rust-js');

async function main() {
    // Create TCP client
    const client = rsocket.createClient({
        type: 'tcp',
        address: '127.0.0.1:7878'
    });
    
    // Request-Response
    const response = await rsocket.requestResponse(client, {
        data: 'Hello, RSocket!'
    });
    
    console.log('Response:', response.data);
    
    // Fire-and-Forget
    await rsocket.fireAndForget(client, {
        data: 'Fire and forget message'
    });
}

main().catch(console.error);
```

## API Reference

### Functions

- `createClient(config)`: Create RSocket client with transport configuration
- `requestResponse(client, payload)`: Send request and wait for response
- `fireAndForget(client, payload)`: Send message without expecting response

### Transport Configuration

```javascript
// TCP Transport
const tcpConfig = {
    type: 'tcp',
    address: '127.0.0.1:7878'
};

// WebSocket Transport
const wsConfig = {
    type: 'websocket',
    address: 'ws://localhost:7879'
};

// QUIC Transport
const quicConfig = {
    type: 'quic',
    address: '127.0.0.1:7880'
};

// Iroh P2P Transport
const irohConfig = {
    type: 'iroh',
    address: 'iroh://peer-id'
};
```

## Development

```bash
# Build the native module
npm run build

# Run tests
npm test
```

## License

Apache-2.0
