.PHONY: build test clean install

# Install dependencies
install:
	npm install

# Build the native module
build:
	npm run build-release

# Test JavaScript bindings
test: build
	npm test

# Clean build artifacts
clean:
	cargo clean
	rm -f index.node

all: install build test
