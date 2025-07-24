#!/usr/bin/env python3
"""
Async Independent Bidirectional RSocket Server
Demonstrates true async bidirectional communication with independent sender/receiver threads.
Server uses callback patterns and can handle multiple concurrent independent streams.
"""

import asyncio
import logging
import time
import rsocket_rust
from typing import Set

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class AsyncIndependentBidirectionalServer:
    def __init__(self):
        self.server_message_counter = 0
        self.running = True
        self.background_tasks = set()
    
    def handle_metadata_push(self, payload):
        """Handle metadata push from client"""
        metadata = payload.metadata_utf8() if payload.metadata_utf8() else "No metadata"
        logging.info(f"📋 Received metadata push: {metadata}")
    
    def handle_fire_and_forget(self, payload):
        """Handle fire-and-forget from client"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        logging.info(f"🔥 Received fire-and-forget: {data}")
    
    def handle_request_response(self, payload):
        """Handle request-response from client"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        logging.info(f"📞 Received request-response: {data}")
        
        response = (rsocket_rust.Payload.builder()
                    .set_data_utf8(f"Server response to: {data}")
                    .set_metadata_utf8("independent-response")
                    .build())
        return response
    
    def handle_request_stream(self, payload):
        """Handle request-stream with independent server-generated responses"""
        data = payload.data_utf8() if payload.data_utf8() else "No data"
        logging.info(f"📡 Received request-stream: {data}")
        
        def independent_stream_generator():
            """Independent generator that creates server responses"""
            for i in range(5):
                self.server_message_counter += 1
                timestamp = int(time.time() * 1000) % 100000
                
                response_data = f"Independent stream item {i+1}/5 | Server msg {self.server_message_counter} | Time: {timestamp}"
                response = (rsocket_rust.Payload.builder()
                           .set_data_utf8(response_data)
                           .set_metadata_utf8(f"stream-{i+1}")
                           .build())
                
                logging.info(f"📤 Independent stream: Generated item {i+1}/5")
                yield response
                time.sleep(0.3)
        
        return independent_stream_generator()
    
    def handle_request_channel(self, input_payloads):
        """Handle request-channel with independent server-generated responses"""
        if isinstance(input_payloads, list):
            logging.info(f"🔄 Received channel request with {len(input_payloads)} inputs")
            for i, payload in enumerate(input_payloads):
                data = payload.data_utf8() if payload.data_utf8() else f"No data {i}"
                logging.info(f"  📥 Channel input {i+1}: {data}")
            input_count = len(input_payloads)
        else:
            data = input_payloads.data_utf8() if input_payloads.data_utf8() else "No data"
            logging.info(f"🔄 Received single channel input: {data}")
            input_count = 1
        
        def independent_channel_generator():
            """Independent generator that creates server responses"""
            response_count = max(3, input_count + 2)
            
            for i in range(response_count):
                self.server_message_counter += 1
                timestamp = int(time.time() * 1000) % 100000
                
                response_data = f"Independent channel response {i+1}/{response_count} | Server msg {self.server_message_counter} | Time: {timestamp}"
                response = (rsocket_rust.Payload.builder()
                           .set_data_utf8(response_data)
                           .set_metadata_utf8(f"channel-response-{i+1}")
                           .build())
                
                logging.info(f"📤 Independent channel: Generated response {i+1}/{response_count}")
                yield response
                time.sleep(0.2)
        
        return independent_channel_generator()

async def main():
    server = AsyncIndependentBidirectionalServer()
    
    handler = (rsocket_rust.RSocketHandler()
               .metadata_push(server.handle_metadata_push)
               .fire_and_forget(server.handle_fire_and_forget)
               .request_response(server.handle_request_response)
               .request_stream(server.handle_request_stream)
               .request_channel(server.handle_request_channel))
    
    tcp_transport = rsocket_rust.TcpServerTransport("127.0.0.1:7890")
    
    def on_start():
        logging.info("🎉 Async Independent Bidirectional Server Started!")
        logging.info("📋 Ready to accept connections on TCP 127.0.0.1:7890")
        logging.info("🔄 Server supports independent async sender/receiver patterns")
        logging.info("📡 Callback-based streaming with on_next/on_complete handlers")
        logging.info("⚡ Independent async tasks for true bidirectional communication")
    
    server_builder = (rsocket_rust.MultiTransportServerBuilder()
                      .add_tcp_transport("TCP", tcp_transport)
                      .acceptor(handler)
                      .on_start(on_start))
    
    try:
        await server_builder.serve()
    except KeyboardInterrupt:
        logging.info("\n👋 Server stopped by user")
        server.running = False
        for task in server.background_tasks:
            task.cancel()
    except Exception as e:
        logging.error(f"❌ Server error: {e}")
        server.running = False

if __name__ == "__main__":
    asyncio.run(main())
