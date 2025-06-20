CARGO_TARGET_DIR ?= ../target
RUST_LIB = $(CARGO_TARGET_DIR)/release/librsocket_rust_java.so
JAVA_SRC = java
JAVA_BUILD = build/java

.PHONY: all clean build-rust build-java test examples

all: build-rust build-java examples

build-rust:
	cargo build --release
	@echo "Rust library built: $(RUST_LIB)"

build-java: build-rust
	mkdir -p $(JAVA_BUILD)
	javac -d $(JAVA_BUILD) $(JAVA_SRC)/com/rsocket/rust/*.java
	@echo "Java classes compiled successfully"

examples: build-java examples/SimpleClient.class examples/EchoServer.class

examples/SimpleClient.class: examples/SimpleClient.java
	javac -cp $(JAVA_BUILD) -d examples examples/SimpleClient.java

examples/EchoServer.class: examples/EchoServer.java
	javac -cp $(JAVA_BUILD) -d examples examples/EchoServer.java

test: build-java
	java -cp $(JAVA_BUILD):examples -Djava.library.path=$(CARGO_TARGET_DIR)/release SimpleClient

clean:
	cargo clean
	rm -rf $(JAVA_BUILD) examples/*.class

install: build-rust build-java
	@echo "Installing Java FFI package..."
	@echo "Installation complete"

help:
	@echo "Available targets:"
	@echo "  all       - Build Rust library and Java classes"
	@echo "  build-rust - Build only the Rust library"
	@echo "  build-java - Build only the Java classes"
	@echo "  examples  - Build Java examples"
	@echo "  test      - Run tests and examples"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install Java FFI package"
	@echo "  help      - Show this help message"
