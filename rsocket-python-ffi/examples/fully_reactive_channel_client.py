#!/usr/bin/env python3
"""
Fully Reactive Channel Client
Demonstrates both reactive input (generators) and reactive output (callbacks)
for truly reactive bidirectional streaming.
"""

import asyncio
import time
import rsocket_rust

class FullyReactiveChannelClient:
    def __init__(self, client):
        self.client = client
        self.responses_received = []
        self.start_time = time.time()
        self.input_count = 0
        self.response_count = 0
    
    def create_payload_generator(self, input_data_list):
        """
        Create a Python generator that yields Payload objects progressively
        This demonstrates reactive input generation
        """
        def payload_generator():
            for i, data in enumerate(input_data_list):
                self.input_count += 1
                print(f"🔧 Generating input {self.input_count}: {data}")
                payload = (rsocket_rust.Payload.builder()
                           .set_data_utf8(data)
                           .build())
                yield payload
                time.sleep(0.15)  # Progressive generation delay
        
        return payload_generator()
    
    def on_response_received(self, payload, response_index):
        """
        Callback for processing each response as it arrives (reactive output)
        """
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        data = payload.data_utf8()
        print(f"📥 REACTIVE Response {response_index} at {elapsed:.1f}ms: {data}")
        
        self.responses_received.append({
            'index': response_index,
            'data': data,
            'timestamp': current_time
        })
        
    
    def on_channel_complete(self, total_responses, success, error):
        """
        Callback when channel communication completes
        """
        if success:
            print(f"✅ Channel completed successfully with {total_responses} responses")
        else:
            print(f"❌ Channel completed with error: {error}")
    
    async def send_fully_reactive_channel(self, input_data_list):
        """
        Send channel with both reactive input (generator) and reactive output (callbacks)
        This uses the new request_channel_reactive_streaming method
        """
        print("🌊 Starting FULLY REACTIVE channel communication...")
        print("   📤 Reactive Input: Generator-based progressive sending")
        print("   📥 Reactive Output: Callback-based progressive processing")
        print()
        
        payload_generator = self.create_payload_generator(input_data_list)
        
        print("🚀 Sending fully reactive channel request...")
        
        total_responses = await self.client.request_channel_reactive_streaming(
            payload_generator,
            self.on_response_received,
            self.on_channel_complete
        )
        
        return total_responses

async def test_fully_reactive_vs_others():
    """Test fully reactive approach vs other approaches"""
    print("🧪 Testing Channel Communication Approaches")
    print("=" * 80)
    
    try:
        print("🔗 Connecting to reactive server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7881")
        )
        print("✅ Connected successfully!")
        print()
        
        input_data = [
            "Fully reactive 1: Hello",
            "Fully reactive 2: World", 
            "Fully reactive 3: RSocket",
            "Fully reactive 4: Channel",
            "Fully reactive 5: Streaming"
        ]
        
        print("📦 Testing BATCH approach (baseline)...")
        start_time = time.time()
        
        input_payloads = []
        for i, data in enumerate(input_data):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(data.replace("Fully reactive", "Batch"))
                       .build())
            input_payloads.append(payload)
        
        batch_responses = await client.request_channel(input_payloads)
        batch_time = time.time() - start_time
        
        print(f"📊 Batch Results: {len(batch_responses)} responses in {batch_time:.3f} seconds")
        print()
        
        await asyncio.sleep(1)
        
        print("🌊 Testing REACTIVE INPUT approach...")
        start_time = time.time()
        
        def simple_generator():
            for data in input_data:
                payload = (rsocket_rust.Payload.builder()
                           .set_data_utf8(data.replace("Fully reactive", "Reactive input"))
                           .build())
                yield payload
                time.sleep(0.1)
        
        reactive_input_responses = await client.request_channel_reactive(simple_generator())
        reactive_input_time = time.time() - start_time
        
        print(f"📊 Reactive Input Results: {len(reactive_input_responses)} responses in {reactive_input_time:.3f} seconds")
        print()
        
        await asyncio.sleep(1)
        
        print("🌊🔄 Testing FULLY REACTIVE approach...")
        reactive_client = FullyReactiveChannelClient(client)
        start_time = time.time()
        
        fully_reactive_responses = await reactive_client.send_fully_reactive_channel(input_data)
        fully_reactive_time = time.time() - start_time
        
        print(f"\n📊 Fully Reactive Results: {fully_reactive_responses} responses in {fully_reactive_time:.3f} seconds")
        
        print("\n" + "=" * 80)
        print("📊 APPROACH COMPARISON SUMMARY:")
        print(f"📦 Batch: {len(batch_responses)} responses in {batch_time:.3f}s (all at once)")
        print(f"🌊 Reactive Input: {len(reactive_input_responses)} responses in {reactive_input_time:.3f}s (progressive input)")
        print(f"🌊🔄 Fully Reactive: {fully_reactive_responses} responses in {fully_reactive_time:.3f}s (progressive input + output)")
        print()
        print("🎯 Key Benefits of Fully Reactive:")
        print("   📤 Progressive input generation with backpressure")
        print("   📥 Real-time response processing as they arrive")
        print("   🔄 True bidirectional streaming")
        print("   💾 Memory efficient for large datasets")
        print("   ⚡ Lower latency for first response")
        print("=" * 80)
        
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()

async def main():
    """Main test function"""
    print("Waiting for server to be ready...")
    await asyncio.sleep(1)
    await test_fully_reactive_vs_others()

if __name__ == "__main__":
    asyncio.run(main())
