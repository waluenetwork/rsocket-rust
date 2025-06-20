CARGO_TARGET_DIR ?= ../target
RUST_LIB = $(CARGO_TARGET_DIR)/release/librsocket_rust_c.a
HEADER_FILE = $(CARGO_TARGET_DIR)/include/rsocket_rust_c_ffi.h

CC = gcc
CXX = g++
CFLAGS = -Wall -Wextra -std=c99 -I$(CARGO_TARGET_DIR)/include
CXXFLAGS = -Wall -Wextra -std=c++14 -I$(CARGO_TARGET_DIR)/include
LDFLAGS = $(CARGO_TARGET_DIR)/release/librsocket_rust_c.a -lpthread -ldl -lm -lssl -lcrypto

.PHONY: all clean build-rust examples test

all: build-rust examples

build-rust:
	cargo build --release
	@echo "Rust library built: $(RUST_LIB)"
	@echo "C header generated: $(HEADER_FILE)"

examples: build-rust examples/simple_client examples/cpp_client examples/performance_benchmark examples/transport_demo

examples/simple_client: examples/simple_client.c $(RUST_LIB)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

examples/cpp_client: examples/cpp_client.cpp $(RUST_LIB)
	$(CXX) $(CXXFLAGS) -o $@ $< $(LDFLAGS)

examples/performance_benchmark: examples/performance_benchmark.c $(RUST_LIB)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

examples/transport_demo: examples/transport_demo.c $(RUST_LIB)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

test: examples
	@echo "Running C client example..."
	@./examples/simple_client || echo "Note: Server must be running for test to succeed"
	@echo "Running C++ client example..."
	@./examples/cpp_client || echo "Note: Server must be running for test to succeed"
	@echo "Running performance benchmark..."
	@./examples/performance_benchmark
	@echo "Running transport demo..."
	@./examples/transport_demo

clean:
	cargo clean
	rm -f examples/simple_client examples/cpp_client examples/performance_benchmark examples/transport_demo

install: build-rust
	@echo "Installing headers and library..."
	sudo mkdir -p /usr/local/include/rsocket
	sudo cp $(HEADER_FILE) /usr/local/include/rsocket/
	sudo cp include/rsocket_cpp.hpp /usr/local/include/rsocket/
	sudo cp $(RUST_LIB) /usr/local/lib/
	sudo ldconfig
	@echo "Installation complete"

help:
	@echo "Available targets:"
	@echo "  all       - Build Rust library and examples"
	@echo "  build-rust - Build only the Rust library"
	@echo "  examples  - Build C/C++ examples"
	@echo "  test      - Run example programs"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install headers and library system-wide"
	@echo "  help      - Show this help message"
