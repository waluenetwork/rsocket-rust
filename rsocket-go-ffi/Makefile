.PHONY: build test clean

# Build the Rust library
build-rust:
	cargo build --release

# Build Go example
build-go: build-rust
	cd example && go build -o example main.go

# Test Go bindings
test: build-rust
	cd example && LD_LIBRARY_PATH=../target/release go run main.go

# Clean build artifacts
clean:
	cargo clean
	rm -f example/example

# Install dependencies
deps:
	go mod tidy

all: deps build-rust build-go test
