.PHONY: build test clean

# Build the Rust library
build:
	cargo build --release

# Build C example
build-example: build
	gcc -o examples/example examples/example.c -L./target/release -lrsocket_c_ffi -lpthread -ldl

# Test C bindings
test: build-example
	LD_LIBRARY_PATH=./target/release ./examples/example

# Clean build artifacts
clean:
	cargo clean
	rm -f examples/example

all: build build-example test
