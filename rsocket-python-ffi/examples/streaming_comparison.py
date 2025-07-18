#!/usr/bin/env python3
"""
Streaming Comparison Example
Compares batch vs callback-based reactive streaming approaches.
"""

import asyncio
import time
import rsocket_rust

class CallbackObserver:
    def __init__(self, name):
        self.name = name
        self.items = []
        self.start_time = time.time()
        self.completed = False
    
    def on_next(self, payload, index):
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        print(f"  📥 [{self.name}] Item {index} at {elapsed:.1f}ms: {payload.data_utf8()}")
        self.items.append({'index': index, 'timestamp': current_time})
    
    def on_complete(self, total_items, success, error):
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        self.completed = True
        if success:
            print(f"  ✅ [{self.name}] Stream completed at {elapsed:.1f}ms ({total_items} items)")
        else:
            print(f"  ❌ [{self.name}] Stream failed at {elapsed:.1f}ms: {error}")

async def test_streaming_comparison():
    """Compare batch vs callback streaming approaches"""
    print("🧪 Streaming Approaches Comparison")
    print("=" * 60)
    
    try:
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7881")
        )
        print("✅ Connected to server")
        print()
        
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8("Comparison test")
                   .build())
        
        print("📦 Test 1: Batch Streaming (current)")
        print("-" * 40)
        
        start_time = time.time()
        responses = await client.request_stream(payload)
        end_time = time.time()
        
        print(f"📥 Received {len(responses)} items in batch")
        print(f"⏱️  Total time: {(end_time - start_time):.3f} seconds")
        print("📋 All items received at once (batch processing)")
        print()
        
        print("🔄 Test 2: Callback Streaming (reactive)")
        print("-" * 40)
        
        observer = CallbackObserver("Callback")
        
        start_time = time.time()
        total_items = await client.request_stream_with_callback(payload, observer.on_next, observer.on_complete)
        end_time = time.time()
        
        print(f"📥 Processed {total_items} items via callbacks")
        print(f"⏱️  Total time: {(end_time - start_time):.3f} seconds")
        print("📋 Items processed individually as they arrived")
        
        print("\n" + "=" * 60)
        print("📊 ANALYSIS")
        print("=" * 60)
        print("Batch Approach:")
        print("  ✅ Simple to use")
        print("  ❌ All items collected before processing")
        print("  ❌ No backpressure control")
        print("  ❌ Memory usage grows with stream size")
        print()
        print("Callback Approach:")
        print("  ✅ True reactive streaming")
        print("  ✅ Items processed as they arrive")
        print("  ✅ Constant memory usage")
        print("  ✅ Supports backpressure")
        print("  ✅ Observable timing between items")
        print("  ✅ Completion callbacks (on_complete event)")
        print(f"  ✅ Stream completion detected: {observer.completed}")
            
    except Exception as e:
        print(f"❌ Comparison error: {e}")

async def main():
    await asyncio.sleep(2)
    await test_streaming_comparison()

if __name__ == "__main__":
    asyncio.run(main())
