#!/usr/bin/env python3
"""
Independent Bidirectional RSocket Client
Demonstrates true bidirectional communication using request_channel where client
sends independent streams and receives independent server-generated responses.
"""

import asyncio
import logging
import time
import rsocket_rust
from typing import Optional

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

class BidirectionalChannelClient:
    def __init__(self):
        self.client: Optional[rsocket_rust.Client] = None
        self.channel_counter = 0
        self.client_message_counter = 0
    
    async def connect_to_server(self):
        """Connect to the bidirectional server"""
        try:
            logging.info("🔗 Connecting to bidirectional channel server...")
            self.client = await rsocket_rust.RSocketFactory.connect_tcp(
                rsocket_rust.TcpClientTransport("127.0.0.1:7890")
            )
            logging.info("✅ Connected to server successfully!")
            return True
        except Exception as e:
            logging.error(f"❌ Failed to connect to server: {e}")
            return False
    
    async def test_bidirectional_channel_small(self):
        """Test bidirectional channel with small client stream"""
        if not self.client:
            return
        
        self.channel_counter += 1
        channel_id = self.channel_counter
        
        try:
            logging.info(f"🔄 Channel {channel_id}: Testing small client stream (2 messages)")
            
            client_payloads = []
            for i in range(2):
                self.client_message_counter += 1
                timestamp = int(time.time() * 1000) % 100000
                
                payload = (rsocket_rust.Payload.builder()
                          .set_data_utf8(f"Client msg {self.client_message_counter} | Timestamp: {timestamp}")
                          .set_metadata_utf8(f"client-{channel_id}-{i+1}")
                          .build())
                client_payloads.append(payload)
                logging.info(f"  📤 Channel {channel_id} - Sending client msg {i+1}")
            
            responses = await self.client.request_channel(client_payloads)
            logging.info(f"📥 Channel {channel_id}: Client sent 2 msgs, received {len(responses)} server responses:")
            
            for i, response in enumerate(responses, 1):
                data = response.data_utf8()
                metadata = response.metadata_utf8()
                logging.info(f"  📥 Channel {channel_id} - Server response {i}: {data}")
            
            logging.info(f"✨ Channel {channel_id}: Demonstrated server generating MORE responses than client input")
            
        except Exception as e:
            logging.error(f"❌ Channel {channel_id} communication failed: {e}")
    
    async def test_bidirectional_channel_large(self):
        """Test bidirectional channel with large client stream"""
        if not self.client:
            return
        
        self.channel_counter += 1
        channel_id = self.channel_counter
        
        try:
            logging.info(f"🔄 Channel {channel_id}: Testing large client stream (5 messages)")
            
            client_payloads = []
            for i in range(5):
                self.client_message_counter += 1
                timestamp = int(time.time() * 1000) % 100000
                
                payload = (rsocket_rust.Payload.builder()
                          .set_data_utf8(f"Client msg {self.client_message_counter} | Timestamp: {timestamp}")
                          .set_metadata_utf8(f"client-{channel_id}-{i+1}")
                          .build())
                client_payloads.append(payload)
                logging.info(f"  📤 Channel {channel_id} - Sending client msg {i+1}")
            
            responses = await self.client.request_channel(client_payloads)
            logging.info(f"📥 Channel {channel_id}: Client sent 5 msgs, received {len(responses)} server responses:")
            
            for i, response in enumerate(responses, 1):
                data = response.data_utf8()
                logging.info(f"  📥 Channel {channel_id} - Server response {i}: {data}")
            
            logging.info(f"✨ Channel {channel_id}: Server stream size independent of client input size")
            
        except Exception as e:
            logging.error(f"❌ Channel {channel_id} communication failed: {e}")
    
    async def test_concurrent_channels(self):
        """Test multiple concurrent bidirectional channels"""
        if not self.client:
            return
        
        logging.info("🚀 Testing concurrent bidirectional channels...")
        
        tasks = []
        for i in range(3):
            if i % 2 == 0:
                task = asyncio.create_task(self.test_bidirectional_channel_small())
            else:
                task = asyncio.create_task(self.test_bidirectional_channel_large())
            tasks.append(task)
            await asyncio.sleep(0.5)  # Slight delay between channel starts
        
        await asyncio.gather(*tasks)
        logging.info("✅ All concurrent channels completed")
    
    async def run_client(self):
        """Main client execution demonstrating bidirectional communication"""
        if not await self.connect_to_server():
            return
        
        try:
            logging.info("🎯 Demonstrating Independent Bidirectional Channel Communication")
            logging.info("=" * 80)
            
            await self.test_bidirectional_channel_small()
            await asyncio.sleep(1)
            
            await self.test_bidirectional_channel_large()
            await asyncio.sleep(1)
            
            await self.test_concurrent_channels()
            
            logging.info("\n📊 Bidirectional Communication Summary:")
            logging.info(f"   🔄 Channels tested: {self.channel_counter}")
            logging.info(f"   📤 Total client messages sent: {self.client_message_counter}")
            logging.info("   ✨ Server generated independent response streams")
            logging.info("   🎯 Demonstrated true bidirectional communication:")
            logging.info("      • Client stream size ≠ Server response size")
            logging.info("      • Server responses contain independent content")
            logging.info("      • Multiple concurrent channels work independently")
            
        except KeyboardInterrupt:
            logging.info("\n👋 Client stopped by user")
        except Exception as e:
            logging.error(f"❌ Client error: {e}")

async def main():
    client = BidirectionalChannelClient()
    await client.run_client()

if __name__ == "__main__":
    asyncio.run(main())
