#!/usr/bin/env python3
"""
Python FFI Reactive Channel Server Example
Demonstrates a comprehensive reactive channel server implementation using Python FFI bindings.

This example shows:
- PyRSocketHandler with request_channel implementation
- PyMultiTransportServerBuilder for TCP transport setup
- Reactive processing of incoming payload streams
- Progressive response generation using Python generators
- Proper acceptor configuration and server lifecycle management
"""

import asyncio
import time
import rsocket_rust

class ReactiveChannelProcessor:
    """
    Reactive channel processor that handles incoming payload streams
    and generates responses progressively using Python generators.
    """
    
    def __init__(self, name="ReactiveChannel"):
        self.name = name
        self.processed_count = 0
        self.start_time = time.time()
    
    def process_individual_payload_async(self, input_payload):
        """
        Process individual payload in separate async task - truly reactive processing.
        
        This function is called once per incoming payload, each in its own async task.
        No bulk processing - each payload triggers immediate individual processing.
        
        Args:
            input_payload: Single Payload object from client
            
        Returns:
            Generator that yields response Payload objects progressively
        """
        self.processed_count += 1
        
        input_data = input_payload.data_utf8() if input_payload.data_utf8() else f"No data {self.processed_count}"
        input_metadata = input_payload.metadata_utf8() if input_payload.metadata_utf8() else "no-metadata"
        
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        print(f"  🚀 [{self.name}] INDIVIDUAL ASYNC Processing payload {self.processed_count} at {elapsed:.1f}ms")
        print(f"      Input: {input_data}")
        print(f"      ⚡ Each payload processed in separate async task - NO BULK PROCESSING!")
        
        def individual_response_generator():
            """Generator for individual payload processing - truly reactive"""
            for i in range(2):
                response_time = time.time()
                response_elapsed = (response_time - self.start_time) * 1000
                
                response_data = f"INDIVIDUAL ASYNC: {input_data} | Response {i+1}/2 | Task: {self.processed_count} | Time: {response_elapsed:.1f}ms"
                response_metadata = f"individual-async-{self.processed_count}-{i+1}|{input_metadata}"
                
                response = (rsocket_rust.Payload.builder()
                           .set_data_utf8(response_data)
                           .set_metadata_utf8(response_metadata)
                           .build())
                
                print(f"    ⚡ [{self.name}] INDIVIDUAL ASYNC Yielding response {i+1}/2 for payload {self.processed_count} at {response_elapsed:.1f}ms")
                yield response
                
                time.sleep(0.05)
        
        return individual_response_generator()

def create_reactive_channel_handler():
    """
    Creates and configures a reactive channel handler using PyRSocketHandler.
    
    Returns:
        Configured RSocketHandler with request_channel implementation
    """
    processor = ReactiveChannelProcessor("ServerProcessor")
    
    def handle_metadata_push(payload):
        """Handle metadata push requests"""
        metadata = payload.metadata_utf8() if payload.metadata_utf8() else "No metadata"
        print(f"📋 Metadata Push: {metadata}")
    
    def handle_fire_and_forget(payload):
        """Handle fire-and-forget requests"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        print(f"🔥 Fire and Forget: {data}")
    
    def handle_request_response(payload):
        """Handle request-response requests"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        print(f"📞 Request-Response: {data}")
        
        response = (rsocket_rust.Payload.builder()
                   .set_data_utf8(f"Echo: {data}")
                   .set_metadata_utf8("response")
                   .build())
        return response
    
    def handle_request_stream(payload):
        """Handle request-stream requests with reactive generator"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        print(f"📡 Request-Stream: {data}")
        
        def stream_generator():
            """Generator for reactive streaming responses"""
            for i in range(3):
                print(f"  📤 Stream item {i+1}")
                response = (rsocket_rust.Payload.builder()
                           .set_data_utf8(f"Stream item {i+1}: {data}")
                           .set_metadata_utf8(f"stream-{i+1}")
                           .build())
                yield response
                time.sleep(0.05)
        
        return stream_generator()
    
    def handle_request_channel(input_payload):
        """
        Main reactive channel handler - processes individual incoming payload
        and returns generator for progressive response streaming.
        
        This is the core functionality demonstrating true reactive channel processing.
        """
        return processor.process_individual_payload_async(input_payload)
    
    handler = (rsocket_rust.RSocketHandler()
               .metadata_push(handle_metadata_push)
               .fire_and_forget(handle_fire_and_forget)
               .request_response(handle_request_response)
               .request_stream(handle_request_stream)
               .request_channel(handle_request_channel))
    
    return handler

async def main():
    """
    Main server function demonstrating PyMultiTransportServerBuilder
    with TCP transport and reactive channel handler configuration.
    """
    print("🚀 Starting Python FFI Reactive Channel Server")
    print("🌊 Demonstrating TRUE REACTIVE channel processing")
    print("📡 Each payload processed individually as it arrives")
    print("⚡ No bulk processing - each payload triggers separate async task")
    print("=" * 80)
    
    try:
        handler = create_reactive_channel_handler()
        print("✅ Created reactive channel handler")
        
        tcp_transport = rsocket_rust.TcpServerTransport("127.0.0.1:7882")
        print("✅ Created TCP transport on 127.0.0.1:7882")
        
        def on_start():
            print("🎉 Reactive Channel Server Started!")
            print("📋 Server Configuration:")
            print("   • Transport: TCP on 127.0.0.1:7882")
            print("   • Handler: Full RSocket pattern support")
            print("   • Channel: TRUE REACTIVE processing - each payload in separate task")
            print("   • Generator: Each payload returns generator for progressive responses")
            print("   • Individual: Each payload processed individually, no bulk processing")
            print("   • FFI: Python-Rust integration via PyO3")
            print()
            print("🧪 Test with reactive channel clients:")
            print("   python examples/reactive_channel_client.py")
            print("   python examples/fully_reactive_channel_client.py")
            print("   python examples/callback_channel_client.py")
            print()
            print("🔄 Use Ctrl+C to stop the server")
            print("=" * 80)
        
        server = (rsocket_rust.MultiTransportServerBuilder()
                  .add_tcp_transport("TCP", tcp_transport)
                  .acceptor(handler)
                  .on_start(on_start))
        
        print("✅ Configured PyMultiTransportServerBuilder with:")
        print("   • TCP transport acceptor")
        print("   • Reactive channel handler")
        print("   • Server lifecycle callbacks")
        print()
        print("🚀 Starting server...")
        
        await server.serve()
        
    except KeyboardInterrupt:
        print("\n👋 Reactive Channel Server stopped by user")
        print("✅ Server shutdown completed")
    except Exception as e:
        print(f"❌ Server error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    print("Python FFI Reactive Channel Server Example")
    print("Demonstrates: PyRSocketHandler + PyMultiTransportServerBuilder + TCP + request_channel")
    print()
    asyncio.run(main())
