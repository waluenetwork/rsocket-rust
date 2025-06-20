# RSocket Go FFI Bindings

Go bindings for the RSocket Rust implementation, providing high-performance reactive streaming capabilities with idiomatic Go APIs.

## Features

- **All RSocket Interaction Patterns**: Request-Response, Fire-and-Forget, Request-Stream, Request-Channel
- **Multiple Transport Types**: TCP, WebSocket, QUIC (Quinn), Iroh P2P
- **Idiomatic Go APIs**: Error handling and patterns that feel natural to Go developers
- **High Performance**: Built on Rust with CGO for maximum performance
- **Concurrent Safe**: Thread-safe client implementation

## Installation

```bash
# Build the Rust library first
cargo build --release

# Then use in your Go project
go mod init your-project
# Copy rsocket.go to your project or use as a module
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"
)

func main() {
    // Create TCP client
    client, err := NewRSocketClient(TCP, "127.0.0.1:7878")
    if err != nil {
        log.Fatal("Failed to create client:", err)
    }
    defer client.Close()
    
    // Request-Response
    response, err := client.RequestResponse("Hello, RSocket!")
    if err != nil {
        log.Fatal("Request failed:", err)
    }
    
    fmt.Println("Response:", response)
    
    // Fire-and-Forget
    err = client.FireAndForget("Fire and forget message")
    if err != nil {
        log.Fatal("Fire and forget failed:", err)
    }
    
    fmt.Println("Fire and forget sent successfully")
}
```

## API Reference

### Types

```go
type TransportType int

const (
    TCP TransportType = iota
    WebSocket
    QUIC
    Iroh
)

type RSocketClient struct {
    // private fields
}
```

### Functions

```go
// Create new RSocket client
func NewRSocketClient(transportType TransportType, address string) (*RSocketClient, error)

// Send request and wait for response
func (c *RSocketClient) RequestResponse(data string) (string, error)

// Send message without expecting response
func (c *RSocketClient) FireAndForget(data string) error

// Close the client and free resources
func (c *RSocketClient) Close()
```

### Transport Examples

```go
// TCP Transport
client, err := NewRSocketClient(TCP, "127.0.0.1:7878")

// WebSocket Transport
client, err := NewRSocketClient(WebSocket, "ws://localhost:7879")

// QUIC Transport
client, err := NewRSocketClient(QUIC, "127.0.0.1:7880")

// Iroh P2P Transport
client, err := NewRSocketClient(Iroh, "iroh://peer-id")
```

## Building

```bash
# Build the Rust library
cargo build --release

# Build Go application
go build -o example main.go

# Run with library path
LD_LIBRARY_PATH=./target/release ./example
```

## Development

```bash
# Test the Go bindings
go test

# Format Go code
go fmt

# Run Go vet
go vet
```

## License

Apache-2.0
