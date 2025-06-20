# RSocket Java FFI

Java FFI bindings for the RSocket Rust implementation, providing high-performance reactive streaming capabilities for Java applications.

## Features

- **Complete RSocket Support**: All RSocket patterns (request-response, fire-and-forget, request-stream, request-channel)
- **Multiple Transports**: TCP and WebSocket with extensible design for future transports
- **Java-idiomatic API**: Clean Java interfaces with proper exception handling and resource management
- **High Performance**: Direct JNI integration with minimal overhead
- **Thread Safety**: Concurrent operations with CompletableFuture-based async support
- **Automatic Memory Management**: Java finalizers and AutoCloseable for automatic cleanup

## Requirements

- Java 8 or higher
- Rust toolchain (for building the native library)
- JNI development headers

## Building

### Build Rust Library
```bash
cargo build --release
```

### Build Java Classes
```bash
make build-java
```

### Build Everything
```bash
make all
```

## Usage

### Initialize RSocket
```java
import com.rsocket.rust.*;

// Initialize the library
RSocket.init();
System.out.println("RSocket version: " + RSocket.getVersion());
```

### Client Example
```java
try (RSocketClient client = new RSocketClient()) {
    // Connect via TCP
    client.connectTcp("127.0.0.1:7878");
    
    // Create payload
    try (Payload payload = Payload.fromString("Hello from Java!", "")) {
        // Send request and get response
        CompletableFuture<Payload> future = client.requestResponse(payload);
        
        try (Payload response = future.get()) {
            System.out.println("Response: " + response.getDataAsString());
        }
    }
}
```

### Server Example
```java
try (RSocketServer server = new RSocketServer("127.0.0.1:7878")) {
    server.startTcp(new RSocketServer.RequestHandler() {
        @Override
        public void onRequest(Payload request) {
            System.out.println("Received: " + request.getDataAsString());
        }
    });
    
    System.out.println("Server started. Press Enter to stop...");
    System.in.read();
}
```

## Examples

- `SimpleClient.java` - Basic client usage
- `EchoServer.java` - Server implementation
- `PerformanceBenchmark.java` - Performance testing

## API Reference

### RSocket
- `static int init()` - Initialize the library
- `static String getVersion()` - Get library version

### Payload
- `static Payload fromString(String data, String metadata)` - Create from strings
- `static Payload fromBytes(byte[] data, byte[] metadata)` - Create from byte arrays
- `String getDataAsString()` - Get data as string
- `byte[] getData()` - Get raw data bytes

### RSocketClient
- `void connectTcp(String address)` - Connect via TCP
- `void connectWebSocket(String url)` - Connect via WebSocket
- `CompletableFuture<Payload> requestResponse(Payload payload)` - Send request
- `void fireAndForget(Payload payload)` - Send fire-and-forget

### RSocketServer
- `void startTcp(RequestHandler handler)` - Start TCP server
- Interface `RequestHandler` with `void onRequest(Payload request)`

## Performance

The Java FFI provides high-performance reactive streaming with:
- Direct JNI calls with minimal overhead
- Zero-copy data transfer where possible
- Concurrent request handling
- Efficient memory management

## License

Apache License 2.0
