#!/usr/bin/env python3
"""
Async Independent Bidirectional RSocket Client
Demonstrates true async bidirectional communication with independent sender/receiver threads.
Uses separate async tasks for sending and callback-based receiving.
"""

import asyncio
import logging
import time
import rsocket_rust
from typing import Optional

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class AsyncIndependentBidirectionalClient:
    def __init__(self):
        self.client: Optional[rsocket_rust.Client] = None
        self.client_message_counter = 0
        self.server_messages_received = 0
        self.running = True
        self.background_tasks = set()
    
    async def connect_to_server(self):
        """Connect to the async bidirectional server"""
        try:
            logging.info("🔗 Connecting to async independent bidirectional server...")
            self.client = await rsocket_rust.RSocketFactory.connect_tcp(
                rsocket_rust.TcpClientTransport("127.0.0.1:7890")
            )
            logging.info("✅ Connected to server successfully!")
            return True
        except Exception as e:
            logging.error(f"❌ Failed to connect to server: {e}")
            return False
    
    def create_stream_callback_receiver(self, stream_name):
        """Create callback receiver for stream responses"""
        
        def on_next(payload, index):
            """Callback for each stream item - runs in separate async context"""
            data = payload.data_utf8() if payload.data_utf8() else "No data"
            self.server_messages_received += 1
            logging.info(f"📥 [{stream_name}] Stream item {index}: {data}")
        
        def on_complete(total_items, success, error):
            """Callback when stream completes"""
            if success:
                logging.info(f"✅ [{stream_name}] Stream completed with {total_items} items")
            else:
                logging.error(f"❌ [{stream_name}] Stream failed: {error}")
        
        return on_next, on_complete
    
    def create_channel_callback_receiver(self, channel_name):
        """Create callback receiver for channel responses"""
        
        def on_response(payload, index):
            """Callback for each channel response - runs in separate async context"""
            data = payload.data_utf8() if payload.data_utf8() else "No data"
            self.server_messages_received += 1
            logging.info(f"📥 [{channel_name}] Channel response {index}: {data}")
        
        def on_complete(total_responses, success, error):
            """Callback when channel completes"""
            if success:
                logging.info(f"✅ [{channel_name}] Channel completed with {total_responses} responses")
            else:
                logging.error(f"❌ [{channel_name}] Channel failed: {error}")
        
        return on_response, on_complete
    
    async def independent_async_sender(self, sender_name, interval=2.0):
        """Independent async sender that sends messages without waiting for responses"""
        try:
            while self.running:
                self.client_message_counter += 1
                timestamp = int(time.time() * 1000) % 100000
                
                fnf_payload = (rsocket_rust.Payload.builder()
                               .set_data_utf8(f"Independent sender {sender_name} msg {self.client_message_counter} | Time: {timestamp}")
                               .set_metadata_utf8(f"sender-{sender_name}")
                               .build())
                
                await self.client.fire_and_forget(fnf_payload)
                logging.info(f"📤 [{sender_name}] Sent independent message {self.client_message_counter}")
                
                await asyncio.sleep(interval)
                
        except Exception as e:
            logging.error(f"❌ Independent sender {sender_name} error: {e}")
    
    async def test_independent_stream_with_callbacks(self):
        """Test independent stream with callback-based receiver"""
        if not self.client:
            return
        
        try:
            logging.info("📡 Testing independent stream with callbacks...")
            
            on_next, on_complete = self.create_stream_callback_receiver("IndependentStream")
            
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8("Independent stream request")
                       .set_metadata_utf8("stream-request")
                       .build())
            
            total_items = await self.client.request_stream_with_callback(
                payload, on_next, on_complete
            )
            
            logging.info(f"📊 Stream request completed, received {total_items} items via callbacks")
            
        except Exception as e:
            logging.error(f"❌ Independent stream test failed: {e}")
    
    async def test_independent_channel_with_callbacks(self):
        """Test independent channel with callback-based receiver"""
        if not self.client:
            return
        
        try:
            logging.info("🔄 Testing independent channel with callbacks...")
            
            on_response, on_complete = self.create_channel_callback_receiver("IndependentChannel")
            
            input_payloads = []
            for i in range(3):
                self.client_message_counter += 1
                timestamp = int(time.time() * 1000) % 100000
                
                payload = (rsocket_rust.Payload.builder()
                          .set_data_utf8(f"Channel input {i+1}/3 | Client msg {self.client_message_counter} | Time: {timestamp}")
                          .set_metadata_utf8(f"channel-input-{i+1}")
                          .build())
                input_payloads.append(payload)
                logging.info(f"📤 Channel input {i+1}: Prepared")
            
            total_responses = await self.client.request_channel_with_callback(
                input_payloads, on_response, on_complete
            )
            
            logging.info(f"📊 Channel request completed, received {total_responses} responses via callbacks")
            
        except Exception as e:
            logging.error(f"❌ Independent channel test failed: {e}")
    
    async def run_independent_bidirectional_test(self):
        """Run complete independent bidirectional test with separate async contexts"""
        if not await self.connect_to_server():
            return
        
        try:
            logging.info("🎯 Starting Independent Async Bidirectional Communication Test")
            logging.info("=" * 80)
            logging.info("🔄 This test demonstrates:")
            logging.info("   • Independent async sender threads using asyncio.create_task")
            logging.info("   • Callback-based receivers with on_next/on_complete")
            logging.info("   • Separate async contexts for send/receive operations")
            logging.info("   • No request-response coupling between sender and receiver")
            logging.info("   • True bidirectional communication with independent timing")
            logging.info("=" * 80)
            
            sender1_task = asyncio.create_task(
                self.independent_async_sender("Sender1", interval=1.5)
            )
            sender2_task = asyncio.create_task(
                self.independent_async_sender("Sender2", interval=2.3)
            )
            
            self.background_tasks.add(sender1_task)
            self.background_tasks.add(sender2_task)
            
            await self.test_independent_stream_with_callbacks()
            await asyncio.sleep(2)
            
            await self.test_independent_channel_with_callbacks()
            await asyncio.sleep(2)
            
            logging.info("⏳ Letting independent senders run for 10 seconds...")
            await asyncio.sleep(10)
            
            logging.info("\n📊 Independent Bidirectional Communication Summary:")
            logging.info(f"   📤 Total client messages sent: {self.client_message_counter}")
            logging.info(f"   📥 Total server messages received: {self.server_messages_received}")
            logging.info("   ✨ Demonstrated independent async sender/receiver threads")
            logging.info("   🔄 Callback-based streaming with on_next/on_complete")
            logging.info("   ⚡ Separate async contexts for true independence")
            
        except KeyboardInterrupt:
            logging.info("\n👋 Client stopped by user")
        except Exception as e:
            logging.error(f"❌ Client error: {e}")
        finally:
            self.running = False
            for task in self.background_tasks:
                task.cancel()

async def main():
    client = AsyncIndependentBidirectionalClient()
    await client.run_independent_bidirectional_test()

if __name__ == "__main__":
    asyncio.run(main())
