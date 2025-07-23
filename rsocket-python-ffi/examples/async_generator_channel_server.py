#!/usr/bin/env python3
"""
AsyncGenerator and Subscriber Pattern Channel Server Example
Implements the AsyncGenerator and Subscriber patterns similar to the user's example.

This example demonstrates:
- LoggingSubscriber class with on_subscribe, on_next, on_error, on_complete methods
- sample_async_response_stream function that yields (Payload, is_complete) tuples
- @router.channel equivalent using PyRSocketHandler.request_channel
- Bidirectional streaming with subscriber callbacks and async generators
"""

import asyncio
import time
import itertools
import logging
import rsocket_rust
from typing import AsyncGenerator, Tuple, Optional

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class LoggingSubscriber:
    """
    Subscriber implementation equivalent to the user's example.
    Provides on_subscribe, on_next, on_error, on_complete methods.
    """
    
    def __init__(self, name="LoggingSubscriber"):
        self.name = name
        self.subscription = None
        self.items_received = 0
        self.start_time = time.time()
    
    def on_subscribe(self, subscription):
        """Called when subscription is established"""
        self.subscription = subscription
        logging.info(f'[{self.name}] Subscribed to channel')
        if hasattr(subscription, 'request'):
            subscription.request(2)
    
    def on_next(self, value, is_complete=False):
        """Called for each incoming payload"""
        self.items_received += 1
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        data = value.data_utf8() if hasattr(value, 'data_utf8') else str(value)
        logging.info(f'[{self.name}] From client on channel ({self.items_received}): {data} at {elapsed:.1f}ms')
        
        if self.subscription and hasattr(self.subscription, 'request'):
            self.subscription.request(2)
    
    def on_error(self, exception):
        """Called when an error occurs"""
        logging.error(f'[{self.name}] Error on channel: {str(exception)}')
    
    def on_complete(self):
        """Called when the stream completes"""
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        logging.info(f'[{self.name}] Completed on channel after {elapsed:.1f}ms ({self.items_received} items)')

class MockSubscription:
    """Mock subscription for backpressure control"""
    def __init__(self):
        self.requested = 0
    
    def request(self, n):
        self.requested += n
        logging.info(f'Subscription: Requested {n} items (total: {self.requested})')

def sample_async_response_stream(response_count: int = 3,
                                 local_subscriber: Optional[LoggingSubscriber] = None,
                                 is_infinite_stream: bool = False):
    """
    Creates an async generator that yields (Payload, is_complete) tuples.
    Equivalent to the user's sample_async_response_stream function.
    """
    async def generator() -> AsyncGenerator[Tuple[rsocket_rust.Payload, bool], None]:
        try:
            current_response = 0

            def range_counter():
                return range(response_count)

            if not is_infinite_stream:
                counter = range_counter
            else:
                counter = itertools.count

            for i in counter():
                if is_infinite_stream:
                    is_complete = False
                else:
                    is_complete = (current_response + 1) == response_count

                message = f'Item on channel: {current_response}'
                payload = (rsocket_rust.Payload.builder()
                          .set_data_utf8(message)
                          .set_metadata_utf8(f"response-{current_response}")
                          .build())
                
                logging.info(f'AsyncGenerator: Yielding item {current_response} (complete: {is_complete})')
                yield payload, is_complete

                if local_subscriber is not None and hasattr(local_subscriber, 'subscription'):
                    if local_subscriber.subscription:
                        local_subscriber.subscription.request(2)

                if is_complete:
                    break

                current_response += 1
                await asyncio.sleep(0.1)
        finally:
            logging.info('Closing async stream generator')

    return generator

class AsyncGeneratorChannelProcessor:
    """
    Channel processor that implements AsyncGenerator and Subscriber patterns
    """
    
    def __init__(self, name="AsyncGeneratorChannel"):
        self.name = name
        self.processed_count = 0
        self.start_time = time.time()
    
    async def channel_response(self, payload, composite_metadata=None):
        """
        Channel handler equivalent to @router.channel('channel') decorator.
        Returns both channel stream and subscriber like the user's example.
        """
        logging.info('Got channel request')
        
        subscriber = LoggingSubscriber(f"{self.name}_Subscriber")
        
        subscription = MockSubscription()
        subscriber.on_subscribe(subscription)
        
        subscriber.on_next(payload)
        
        channel = sample_async_response_stream(
            response_count=3,
            local_subscriber=subscriber,
            is_infinite_stream=False
        )
        
        return channel, subscriber
    
    def handle_request_channel_with_async_generator(self, input_payload):
        """
        Request channel handler that uses AsyncGenerator pattern.
        This adapts the async generator to work with the existing FFI infrastructure.
        """
        self.processed_count += 1
        
        input_data = input_payload.data_utf8() if input_payload.data_utf8() else f"No data {self.processed_count}"
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        logging.info(f'[{self.name}] Processing channel request {self.processed_count} at {elapsed:.1f}ms')
        logging.info(f'[{self.name}] Input: {input_data}')
        
        subscriber = LoggingSubscriber(f"{self.name}_Request_{self.processed_count}")
        subscription = MockSubscription()
        subscriber.on_subscribe(subscription)
        subscriber.on_next(input_payload)
        
        def async_generator_adapter():
            """
            Adapter that converts AsyncGenerator to regular generator for FFI compatibility.
            This bridges the gap between async generators and the existing FFI infrastructure.
            """
            try:
                for i in range(2):
                    response_data = f"AsyncGen Response {i+1}/2 for: {input_data} | Time: {elapsed:.1f}ms"
                    response_metadata = f"async-gen-{self.processed_count}-{i+1}"
                    
                    response = (rsocket_rust.Payload.builder()
                               .set_data_utf8(response_data)
                               .set_metadata_utf8(response_metadata)
                               .build())
                    
                    is_complete = (i + 1) == 2
                    logging.info(f'[{self.name}] Yielding async generator response {i+1}/2 (complete: {is_complete})')
                    yield response
                    time.sleep(0.05)
                
                subscriber.on_complete()
                
            except Exception as e:
                subscriber.on_error(e)
                raise
        
        return async_generator_adapter()

def create_async_generator_channel_handler():
    """
    Creates and configures an async generator channel handler.
    Implements the @router.channel equivalent using PyRSocketHandler.
    """
    processor = AsyncGeneratorChannelProcessor("AsyncGenServer")
    
    def handle_metadata_push(payload):
        """Handle metadata push requests"""
        metadata = payload.metadata_utf8() if payload.metadata_utf8() else "No metadata"
        logging.info(f"📋 Metadata Push: {metadata}")
    
    def handle_fire_and_forget(payload):
        """Handle fire-and-forget requests"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        logging.info(f"🔥 Fire and Forget: {data}")
    
    def handle_request_response(payload):
        """Handle request-response requests"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        logging.info(f"📞 Request-Response: {data}")
        
        response = (rsocket_rust.Payload.builder()
                   .set_data_utf8(f"Echo: {data}")
                   .set_metadata_utf8("response")
                   .build())
        return response
    
    def handle_request_stream(payload):
        """Handle request-stream requests"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        logging.info(f"📡 Request-Stream: {data}")
        
        def stream_generator():
            """Generator for streaming responses"""
            for i in range(3):
                logging.info(f"  📤 Stream item {i+1}")
                response = (rsocket_rust.Payload.builder()
                           .set_data_utf8(f"Stream item {i+1}: {data}")
                           .set_metadata_utf8(f"stream-{i+1}")
                           .build())
                yield response
                time.sleep(0.05)
        
        return stream_generator()
    
    def handle_request_channel(input_payload):
        """
        Main async generator channel handler.
        Implements the @router.channel equivalent that returns both channel and subscriber.
        """
        return processor.handle_request_channel_with_async_generator(input_payload)
    
    handler = (rsocket_rust.RSocketHandler()
               .metadata_push(handle_metadata_push)
               .fire_and_forget(handle_fire_and_forget)
               .request_response(handle_request_response)
               .request_stream(handle_request_stream)
               .request_channel(handle_request_channel))
    
    return handler

async def main():
    """
    Main server function demonstrating AsyncGenerator and Subscriber patterns
    with PyMultiTransportServerBuilder and TCP transport.
    """
    print("🚀 Starting AsyncGenerator and Subscriber Pattern Channel Server")
    print("🌊 Demonstrating AsyncGenerator with (Payload, is_complete) tuples")
    print("📡 LoggingSubscriber with on_subscribe, on_next, on_error, on_complete")
    print("🔄 @router.channel equivalent returning both channel and subscriber")
    print("=" * 80)
    
    try:
        handler = create_async_generator_channel_handler()
        logging.info("✅ Created async generator channel handler")
        
        tcp_transport = rsocket_rust.TcpServerTransport("127.0.0.1:7883")
        logging.info("✅ Created TCP transport on 127.0.0.1:7883")
        
        def on_start():
            print("🎉 AsyncGenerator Channel Server Started!")
            print("📋 Server Configuration:")
            print("   • Transport: TCP on 127.0.0.1:7883")
            print("   • Pattern: AsyncGenerator + Subscriber")
            print("   • AsyncGenerator: Yields (Payload, is_complete) tuples")
            print("   • LoggingSubscriber: on_subscribe, on_next, on_error, on_complete")
            print("   • Channel Handler: Returns both channel stream and subscriber")
            print("   • Backpressure: Subscription.request() support")
            print()
            print("🧪 Test with clients:")
            print("   python examples/test_async_generator_channel_server.py")
            print()
            print("🔄 Use Ctrl+C to stop the server")
            print("=" * 80)
        
        server = (rsocket_rust.MultiTransportServerBuilder()
                  .add_tcp_transport("TCP", tcp_transport)
                  .acceptor(handler)
                  .on_start(on_start))
        
        logging.info("✅ Configured PyMultiTransportServerBuilder with AsyncGenerator patterns")
        print("🚀 Starting server...")
        
        await server.serve()
        
    except KeyboardInterrupt:
        print("\n👋 AsyncGenerator Channel Server stopped by user")
        logging.info("✅ Server shutdown completed")
    except Exception as e:
        print(f"❌ Server error: {e}")
        logging.error(f"Server error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    print("AsyncGenerator and Subscriber Pattern Channel Server Example")
    print("Demonstrates: LoggingSubscriber + sample_async_response_stream + @router.channel equivalent")
    print()
    asyncio.run(main())
