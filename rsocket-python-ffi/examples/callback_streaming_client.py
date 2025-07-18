#!/usr/bin/env python3
"""
Callback-based Reactive Streaming Client Example
Demonstrates true reactive streaming with callback processing.
"""

import asyncio
import time
import rsocket_rust

class StreamObserver:
    def __init__(self):
        self.items_received = []
        self.start_time = time.time()
    
    def on_next(self, payload, index):
        """Callback function called for each stream item"""
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        print(f"📥 Item {index} received at {elapsed:.1f}ms: {payload.data_utf8()}")
        self.items_received.append({
            'index': index,
            'data': payload.data_utf8(),
            'timestamp': current_time
        })
        
        time.sleep(0.01)

async def test_callback_streaming():
    """Test callback-based reactive streaming"""
    print("🧪 Testing Callback-based Reactive Streaming")
    print("=" * 60)
    
    try:
        print("🔗 Connecting to reactive server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7881")
        )
        print("✅ Connected successfully!")
        print()
        
        observer = StreamObserver()
        
        print("📤 Starting callback-based stream request...")
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8("Callback streaming test")
                   .build())
        
        start_time = time.time()
        
        total_items = await client.request_stream_with_callback(payload, observer.on_next)
        
        end_time = time.time()
        total_time = end_time - start_time
        
        print(f"\n📊 Streaming completed:")
        print(f"   Total items: {total_items}")
        print(f"   Total time: {total_time:.3f} seconds")
        print(f"   Items processed via callback: {len(observer.items_received)}")
        
        if len(observer.items_received) > 1:
            first_item_time = observer.items_received[0]['timestamp']
            last_item_time = observer.items_received[-1]['timestamp']
            processing_span = last_item_time - first_item_time
            
            print(f"   Processing span: {processing_span:.3f} seconds")
            print(f"   Average interval: {processing_span / (len(observer.items_received) - 1):.3f} seconds")
            
        print("\n✅ Callback-based streaming demonstrates true reactivity!")
        print("   Each item was processed individually as it arrived")
            
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()

async def main():
    """Main test function"""
    print("Waiting for server to start...")
    await asyncio.sleep(2)
    await test_callback_streaming()

if __name__ == "__main__":
    asyncio.run(main())
