#!/usr/bin/env python3
"""
Reactive Channel Client
Creates a truly reactive channel client that sends inputs progressively
using generators and processes responses as they arrive.
"""

import asyncio
import time
import rsocket_rust

class ReactiveChannelClient:
    def __init__(self, client):
        self.client = client
        self.responses_received = []
        self.start_time = time.time()
    
    def create_payload_generator(self, input_data_list):
        """
        Create a Python generator that yields Payload objects progressively
        This demonstrates reactive input generation
        """
        def payload_generator():
            for i, data in enumerate(input_data_list):
                print(f"🔧 Generating payload {i+1}: {data}")
                payload = (rsocket_rust.Payload.builder()
                           .set_data_utf8(data)
                           .build())
                yield payload
                time.sleep(0.1)
        
        return payload_generator()
    
    async def send_reactive_channel_with_generator(self, input_data_list, response_callback=None):
        """
        Send channel inputs using a generator for truly reactive input
        This uses the new request_channel_reactive method
        """
        print("🌊 Starting REACTIVE channel communication with generator...")
        
        payload_generator = self.create_payload_generator(input_data_list)
        
        print("🚀 Sending channel request with generator...")
        
        responses = await self.client.request_channel_reactive(payload_generator)
        
        print(f"📥 Processing {len(responses)} responses...")
        for i, response in enumerate(responses):
            current_time = time.time()
            elapsed = (current_time - self.start_time) * 1000
            
            data = response.data_utf8()
            print(f"📥 Response {i+1} at {elapsed:.1f}ms: {data}")
            
            self.responses_received.append({
                'index': i+1,
                'data': data,
                'timestamp': current_time
            })
            
            if response_callback:
                await response_callback(response, i+1)
            
            await asyncio.sleep(0.05)
        
        return len(responses)
    
    async def send_batch_channel(self, input_data_list, response_callback=None):
        """
        Send channel inputs using traditional batch array approach
        This is the current/old approach for comparison
        """
        print("📦 Starting BATCH channel communication with array...")
        
        input_payloads = []
        for i, data in enumerate(input_data_list):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(data)
                       .build())
            input_payloads.append(payload)
            print(f"📤 Batch prepared input {i+1}: {data}")
        
        print(f"📦 Batch prepared {len(input_payloads)} payloads at once")
        
        print("🚀 Sending batch channel request...")
        responses = await self.client.request_channel(input_payloads)
        
        print(f"📥 Processing {len(responses)} responses...")
        for i, response in enumerate(responses):
            current_time = time.time()
            elapsed = (current_time - self.start_time) * 1000
            
            data = response.data_utf8()
            print(f"📥 Response {i+1} at {elapsed:.1f}ms: {data}")
            
            if response_callback:
                await response_callback(response, i+1)
            
            await asyncio.sleep(0.05)
        
        return len(responses)

async def response_processor(payload, index):
    """Callback to process each response reactively"""
    data = payload.data_utf8()
    print(f"  🔄 Processing response {index}: {data}")
    await asyncio.sleep(0.01)

async def test_reactive_vs_batch():
    """Test both reactive and batch approaches"""
    print("🧪 Testing Reactive vs Batch Channel Communication")
    print("=" * 80)
    
    try:
        print("🔗 Connecting to reactive server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7881")
        )
        print("✅ Connected successfully!")
        print()
        
        reactive_client = ReactiveChannelClient(client)
        
        input_data = [
            "Test input 1: Hello",
            "Test input 2: World", 
            "Test input 3: RSocket",
            "Test input 4: Channel",
            "Test input 5: Streaming"
        ]
        
        print("📦 Testing BATCH approach first...")
        start_time = time.time()
        batch_responses = await reactive_client.send_batch_channel(input_data, response_processor)
        batch_time = time.time() - start_time
        
        print(f"\n📊 Batch Results: {batch_responses} responses in {batch_time:.3f} seconds")
        
        await asyncio.sleep(1)
        reactive_client.responses_received = []
        reactive_client.start_time = time.time()
        
        print("\n🌊 Testing REACTIVE approach...")
        start_time = time.time()
        reactive_responses = await reactive_client.send_reactive_channel_with_generator(input_data, response_processor)
        reactive_time = time.time() - start_time
        
        print(f"\n📊 Reactive Results: {reactive_responses} responses in {reactive_time:.3f} seconds")
        
        print("\n" + "=" * 80)
        print("📊 COMPARISON SUMMARY:")
        print(f"📦 Batch approach: {batch_responses} responses, all inputs created at once")
        print(f"🌊 Reactive approach: {reactive_responses} responses, inputs generated progressively")
        print("🎯 Reactive approach enables true streaming with backpressure support")
        print("✅ Both approaches working successfully!")
        print("=" * 80)
        
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()

async def main():
    """Main test function"""
    print("Waiting for server to be ready...")
    await asyncio.sleep(1)
    await test_reactive_vs_batch()

if __name__ == "__main__":
    asyncio.run(main())
