#!/usr/bin/env python3
"""
Independent Bidirectional RSocket Server
Demonstrates true bidirectional communication using request_channel where both
client and server can send independent streams of data through the same channel.
"""

import asyncio
import logging
import time
import rsocket_rust

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class BidirectionalChannelServer:
    def __init__(self):
        self.channel_counter = 0
        self.server_message_counter = 0
    
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
                    .set_metadata_utf8("response")
                    .build())
        return response
    
    def handle_request_channel(self, payloads):
        """
        Handle bidirectional channel - server sends independent stream of responses
        that are NOT just echoes of client input, demonstrating true bidirectional flow
        """
        self.channel_counter += 1
        channel_id = self.channel_counter
        
        if isinstance(payloads, list):
            client_count = len(payloads)
            logging.info(f"🔄 Channel {channel_id}: Received {client_count} client messages")
            
            for i, payload in enumerate(payloads):
                data = payload.data_utf8() if payload.data_utf8() else f"No data {i}"
                logging.info(f"  📥 Channel {channel_id} - Client msg {i+1}: {data}")
        else:
            logging.info(f"🔄 Channel {channel_id}: Received single client message")
            data = payloads.data_utf8() if payloads.data_utf8() else "No data"
            logging.info(f"  📥 Channel {channel_id} - Client msg: {data}")
            client_count = 1
        
        server_responses = []
        server_stream_size = max(3, client_count + 2)  # Always send more than client
        
        for i in range(server_stream_size):
            self.server_message_counter += 1
            
            server_data = f"Server-generated message {self.server_message_counter} (Channel {channel_id}, Item {i+1}/{server_stream_size})"
            server_metadata = f"server-stream-{channel_id}-{i+1}"
            
            timestamp = int(time.time() * 1000) % 100000
            server_data += f" | Timestamp: {timestamp}"
            
            response = (rsocket_rust.Payload.builder()
                        .set_data_utf8(server_data)
                        .set_metadata_utf8(server_metadata)
                        .build())
            server_responses.append(response)
            
            logging.info(f"  📤 Channel {channel_id} - Server msg {i+1}: Generated independent content")
        
        logging.info(f"🎯 Channel {channel_id}: Client sent {client_count} msgs, Server responds with {server_stream_size} independent msgs")
        return server_responses

async def main():
    server = BidirectionalChannelServer()
    
    handler = (rsocket_rust.RSocketHandler()
               .metadata_push(server.handle_metadata_push)
               .fire_and_forget(server.handle_fire_and_forget)
               .request_response(server.handle_request_response)
               .request_channel(server.handle_request_channel))
    
    tcp_transport = rsocket_rust.TcpServerTransport("127.0.0.1:7890")
    
    def on_start():
        logging.info("🎉 Bidirectional Channel Server Started!")
        logging.info("📋 Ready to accept connections on TCP 127.0.0.1:7890")
        logging.info("🔄 Server generates independent response streams")
        logging.info("✨ Demonstrates true bidirectional communication")
    
    server_builder = (rsocket_rust.MultiTransportServerBuilder()
                      .add_tcp_transport("TCP", tcp_transport)
                      .acceptor(handler)
                      .on_start(on_start))
    
    try:
        await server_builder.serve()
    except KeyboardInterrupt:
        logging.info("\n👋 Server stopped by user")
    except Exception as e:
        logging.error(f"❌ Server error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
