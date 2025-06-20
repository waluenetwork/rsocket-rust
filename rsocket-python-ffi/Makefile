.PHONY: build test clean install

# Install Python dependencies
install:
	python -m pip install maturin

# Build Python FFI
build:
	maturin develop

# Build with advanced transports
build-advanced:
	maturin develop --features advanced-transports

# Test Python bindings
test: build-advanced
	python examples/enhanced_transports_demo.py

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/
	rm -rf build/
	rm -rf dist/

all: install build-advanced test
