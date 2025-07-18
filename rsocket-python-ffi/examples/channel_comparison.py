#!/usr/bin/env python3
"""
Channel Streaming Comparison Example
Compares batch vs callback-based channel streaming approaches.
"""

import asyncio
import time
import rsocket_rust

class ChannelCallbackObserver:
    def __init__(self, name):
        self.name = name
        self.responses = []
        self.start_time = time.time()
        self.completed = False
    
    def on_response(self, payload, index):
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        print(f"  📥 [{self.name}] Response {index} at {elapsed:.1f}ms: {payload.data_utf8()}")
        self.responses.append({'index': index, 'timestamp': current_time})
    
    def on_complete(self, total_responses, success, error):
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        self.completed = True
        if success:
            print(f"  ✅ [{self.name}] Channel completed at {elapsed:.1f}ms ({total_responses} responses)")
        else:
            print(f"  ❌ [{self.name}] Channel failed at {elapsed:.1f}ms: {error}")

async def test_channel_comparison():
    """Compare batch vs callback channel streaming approaches"""
    print("🧪 Channel Streaming Approaches Comparison")
    print("=" * 60)
    
    try:
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7881")
        )
        print("✅ Connected to server")
        print()
        
        input_payloads = []
        for i in range(1, 4):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"Comparison test {i}")
                       .build())
            input_payloads.append(payload)
        
        print("📦 Test 1: Batch Channel (current)")
        print("-" * 40)
        
        start_time = time.time()
        responses = await client.request_channel(input_payloads)
        end_time = time.time()
        
        print(f"📥 Sent {len(input_payloads)} inputs, received {len(responses)} responses in batch")
        print(f"⏱️  Total time: {(end_time - start_time):.3f} seconds")
        print("📋 All responses received at once (batch processing)")
        print()
        
        print("🔄 Test 2: Callback Channel (reactive)")
        print("-" * 40)
        
        observer = ChannelCallbackObserver("Callback")
        
        start_time = time.time()
        total_responses = await client.request_channel_with_callback(
            input_payloads, observer.on_response, observer.on_complete
        )
        end_time = time.time()
        
        print(f"📥 Sent {len(input_payloads)} inputs, processed {total_responses} responses via callbacks")
        print(f"⏱️  Total time: {(end_time - start_time):.3f} seconds")
        print("📋 Responses processed individually as they arrived")
        
        print("\n" + "=" * 60)
        print("📊 CHANNEL ANALYSIS")
        print("=" * 60)
        print("Batch Channel Approach:")
        print("  ✅ Simple to use")
        print("  ❌ All inputs sent at once")
        print("  ❌ All responses collected before processing")
        print("  ❌ No backpressure control")
        print("  ❌ Memory usage grows with channel size")
        print()
        print("Callback Channel Approach:")
        print("  ✅ True reactive bidirectional streaming")
        print("  ✅ Responses processed as they arrive")
        print("  ✅ Constant memory usage")
        print("  ✅ Supports backpressure")
        print("  ✅ Observable timing between responses")
        print("  ✅ Completion callbacks (on_complete event)")
        print(f"  ✅ Channel completion detected: {observer.completed}")
            
    except Exception as e:
        print(f"❌ Comparison error: {e}")

async def main():
    await asyncio.sleep(2)
    await test_channel_comparison()

if __name__ == "__main__":
    asyncio.run(main())
