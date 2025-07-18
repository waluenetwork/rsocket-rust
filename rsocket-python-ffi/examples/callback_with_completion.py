#!/usr/bin/env python3
"""
Complete Callback-based Streaming Example
Demonstrates on_next and on_complete callbacks with error handling.
"""

import asyncio
import time
import rsocket_rust

class CompleteStreamObserver:
    def __init__(self, name="Observer"):
        self.name = name
        self.items_received = []
        self.start_time = time.time()
        self.completed = False
        self.completion_success = False
        self.completion_error = None
        self.completion_time = None
    
    def on_next(self, payload, index):
        """Called for each stream item"""
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        data = payload.data_utf8()
        print(f"📥 [{self.name}] Item {index} at {elapsed:.1f}ms: {data}")
        
        self.items_received.append({
            'index': index,
            'data': data,
            'timestamp': current_time
        })
        
        time.sleep(0.005)
    
    def on_complete(self, total_items, success, error):
        """Called when stream completes (success or error)"""
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        self.completed = True
        self.completion_time = current_time
        self.completion_success = success is not None and success
        self.completion_error = error
        
        if self.completion_success:
            print(f"✅ [{self.name}] Stream COMPLETED successfully at {elapsed:.1f}ms")
            print(f"   📊 Total items processed: {total_items}")
            print(f"   📊 Items received via callbacks: {len(self.items_received)}")
        else:
            print(f"❌ [{self.name}] Stream FAILED at {elapsed:.1f}ms")
            print(f"   📊 Items processed before failure: {total_items}")
            print(f"   ⚠️  Error: {error}")
    
    def get_summary(self):
        """Get streaming summary"""
        if not self.completed:
            return "Stream not completed yet"
        
        total_time = self.completion_time - self.start_time
        return {
            'completed': self.completed,
            'success': self.completion_success,
            'error': self.completion_error,
            'total_time': total_time,
            'items_received': len(self.items_received),
            'avg_interval': total_time / max(1, len(self.items_received) - 1) if len(self.items_received) > 1 else 0
        }

async def test_complete_callbacks():
    """Test complete callback lifecycle"""
    print("🧪 Testing Complete Callback Lifecycle")
    print("=" * 60)
    
    try:
        print("🔗 Connecting to reactive server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7881")
        )
        print("✅ Connected successfully!")
        print()
        
        print("🔄 Test 1: Normal Stream Completion")
        print("-" * 40)
        
        observer1 = CompleteStreamObserver("Normal")
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8("Complete callback test")
                   .build())
        
        total_items = await client.request_stream_with_callback(
            payload, observer1.on_next, observer1.on_complete
        )
        
        summary1 = observer1.get_summary()
        print(f"📋 Summary: {summary1}")
        print()
        
        print("🔄 Test 2: Without Completion Callback")
        print("-" * 40)
        
        observer2 = CompleteStreamObserver("NoComplete")
        
        total_items2 = await client.request_stream_with_callback(
            payload, observer2.on_next
        )
        
        print(f"📋 Items processed without completion callback: {total_items2}")
        print(f"📋 Observer completion status: {observer2.completed}")
        print()
        
        print("=" * 60)
        print("📊 CALLBACK LIFECYCLE ANALYSIS")
        print("=" * 60)
        print("With Completion Callback:")
        print(f"  ✅ Stream lifecycle fully observable")
        print(f"  ✅ Completion event fired: {summary1['completed']}")
        print(f"  ✅ Success status available: {summary1['success']}")
        print(f"  ✅ Total processing time: {summary1['total_time']:.3f}s")
        print()
        print("Without Completion Callback:")
        print(f"  ⚠️  No completion event (optional parameter)")
        print(f"  ✅ Stream still processes items normally")
        print(f"  ✅ Return value indicates completion")
        
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()

async def main():
    """Main test function"""
    print("Waiting for server to start...")
    await asyncio.sleep(2)
    await test_complete_callbacks()

if __name__ == "__main__":
    asyncio.run(main())
