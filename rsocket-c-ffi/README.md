# RSocket C/C++ FFI Bindings

C/C++ bindings for the RSocket Rust implementation, providing high-performance reactive streaming capabilities with both synchronous and asynchronous APIs.

## Features

- **All RSocket Interaction Patterns**: Request-Response, Fire-and-Forget, Request-Stream, Request-Channel
- **Multiple Transport Types**: TCP, WebSocket, QUIC (Quinn), Iroh P2P
- **Synchronous and Asynchronous APIs**: Choose the best fit for your application
- **C and C++ Compatible**: Works with both C and C++ applications
- **High Performance**: Built on Rust for maximum performance

## Installation

```bash
# Build the library
cargo build --release

# The library will be available at:
# - target/release/librsocket_c_ffi.so (Linux)
# - target/release/librsocket_c_ffi.dylib (macOS)
# - target/release/rsocket_c_ffi.dll (Windows)
```

## Quick Start

### C Example

```c
#include "include/rsocket_c.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void response_callback(const CRSocketPayload* payload, void* user_data) {
    if (payload && payload->data) {
        printf("Async Response: %.*s\n", (int)payload->data_len, payload->data);
    }
}

int main() {
    // Create client configuration
    CRSocketTransportConfig config = {
        .transport_type = 0, // TCP
        .address = "127.0.0.1:7878",
        .enable_advanced_features = 0
    };
    
    // Create client
    CRSocketClient* client = rsocket_c_create_client(&config);
    if (!client) {
        printf("Failed to create client\n");
        return 1;
    }
    
    // Synchronous request-response
    char* response_data;
    size_t response_len;
    const char* message = "Hello, RSocket!";
    
    int result = rsocket_c_request_response_sync(
        client,
        message,
        strlen(message),
        &response_data,
        &response_len
    );
    
    if (result == 0) {
        printf("Sync Response: %.*s\n", (int)response_len, response_data);
        rsocket_go_free_string(response_data);
    }
    
    // Asynchronous request-response
    CRSocketPayload payload = {
        .data = (char*)message,
        .data_len = strlen(message),
        .metadata = NULL,
        .metadata_len = 0
    };
    
    rsocket_c_request_response_async(client, &payload, response_callback, NULL);
    
    // Clean up
    rsocket_c_free_client(client);
    return 0;
}
```

### C++ Example

```cpp
#include "include/rsocket_c.h"
#include <iostream>
#include <string>
#include <memory>

class RSocketClientWrapper {
private:
    CRSocketClient* client;
    
public:
    RSocketClientWrapper(int transport_type, const std::string& address) {
        CRSocketTransportConfig config = {
            .transport_type = transport_type,
            .address = address.c_str(),
            .enable_advanced_features = 0
        };
        
        client = rsocket_c_create_client(&config);
        if (!client) {
            throw std::runtime_error("Failed to create RSocket client");
        }
    }
    
    ~RSocketClientWrapper() {
        if (client) {
            rsocket_c_free_client(client);
        }
    }
    
    std::string requestResponse(const std::string& data) {
        char* response_data;
        size_t response_len;
        
        int result = rsocket_c_request_response_sync(
            client,
            data.c_str(),
            data.length(),
            &response_data,
            &response_len
        );
        
        if (result != 0) {
            throw std::runtime_error("Request-response failed");
        }
        
        std::string response(response_data, response_len);
        rsocket_go_free_string(response_data);
        return response;
    }
};

int main() {
    try {
        RSocketClientWrapper client(0, "127.0.0.1:7878"); // TCP
        
        std::string response = client.requestResponse("Hello from C++!");
        std::cout << "Response: " << response << std::endl;
        
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
    
    return 0;
}
```

## API Reference

### Transport Types

- `0`: TCP
- `1`: WebSocket
- `2`: QUIC
- `3`: Iroh P2P

### Functions

- `rsocket_c_create_client()`: Create RSocket client
- `rsocket_c_request_response_sync()`: Synchronous request-response
- `rsocket_c_request_response_async()`: Asynchronous request-response with callback
- `rsocket_c_free_client()`: Free client resources
- `rsocket_c_free_payload()`: Free payload resources

## Building

```bash
# Build C/C++ library
cargo build --release

# Compile C example
gcc -o example example.c -L./target/release -lrsocket_c_ffi

# Compile C++ example
g++ -o example example.cpp -L./target/release -lrsocket_c_ffi
```

## License

Apache-2.0
