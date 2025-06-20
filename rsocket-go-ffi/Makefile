CARGO_TARGET_DIR ?= ../target
RUST_LIB = $(CARGO_TARGET_DIR)/release/librsocket_rust_go.a
GO_PKG = ./go

.PHONY: all clean build-rust build-go test examples

all: build-rust build-go examples

build-rust:
	cargo build --release
	@echo "Rust library built: $(RUST_LIB)"

build-go: build-rust
	cd $(GO_PKG) && go build -v ./...
	@echo "Go package built successfully"

examples: build-go examples/simple-client examples/echo-server examples/performance-benchmark

examples/simple-client: examples/simple-client.go
	cd examples && go build -o simple-client simple-client.go

examples/echo-server: examples/echo-server.go  
	cd examples && go build -o echo-server echo-server.go

examples/performance-benchmark: examples/performance-benchmark.go
	cd examples && go build -o performance-benchmark performance-benchmark.go

test: build-go
	cd $(GO_PKG) && go test -v ./...
	@echo "Running Go client example..."
	@cd examples && ./simple-client || echo "Note: Server must be running for test to succeed"

clean:
	cargo clean
	cd $(GO_PKG) && go clean
	rm -f examples/simple-client examples/echo-server examples/performance-benchmark

install: build-rust build-go
	@echo "Installing Go package..."
	cd $(GO_PKG) && go install ./...
	@echo "Installation complete"

help:
	@echo "Available targets:"
	@echo "  all       - Build Rust library and Go package"
	@echo "  build-rust - Build only the Rust library"
	@echo "  build-go  - Build only the Go package"
	@echo "  examples  - Build Go examples"
	@echo "  test      - Run tests and examples"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install Go package"
	@echo "  help      - Show this help message"
