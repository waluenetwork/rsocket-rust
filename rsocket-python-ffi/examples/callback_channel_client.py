#!/usr/bin/env python3
"""
Callback-based Channel Streaming Client Example
Demonstrates true reactive bidirectional streaming with callback processing.
"""

import asyncio
import time
import rsocket_rust

class ChannelObserver:
    def __init__(self, name="Channel"):
        self.name = name
        self.responses_received = []
        self.start_time = time.time()
        self.completed = False
        self.completion_error = None
    
    def on_response(self, payload, index):
        """Callback function called for each response item"""
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        data = payload.data_utf8()
        print(f"📥 [{self.name}] Response {index} at {elapsed:.1f}ms: {data}")
        
        self.responses_received.append({
            'index': index,
            'data': data,
            'timestamp': current_time
        })
        
        time.sleep(0.01)
    
    def on_complete(self, total_responses, success, error):
        """Callback function called when channel completes"""
        current_time = time.time()
        elapsed = (current_time - self.start_time) * 1000
        
        self.completed = True
        if success:
            print(f"✅ [{self.name}] Channel completed successfully at {elapsed:.1f}ms")
            print(f"   Total responses processed: {total_responses}")
        else:
            self.completion_error = error
            print(f"❌ [{self.name}] Channel completed with error at {elapsed:.1f}ms: {error}")
            print(f"   Responses processed before error: {total_responses}")

async def test_callback_channel():
    """Test callback-based reactive channel streaming"""
    print("🧪 Testing Callback-based Channel Streaming")
    print("=" * 60)
    
    try:
        print("🔗 Connecting to reactive server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7881")
        )
        print("✅ Connected successfully!")
        print()
        
        observer = ChannelObserver("Reactive")
        
        print("📤 Preparing input payloads for channel...")
        input_payloads = []
        for i in range(1, 6):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"Channel input {i}")
                       .build())
            input_payloads.append(payload)
            print(f"📦 Input {i}: Channel input {i}")
        
        print(f"\n🔄 Starting callback-based channel request with {len(input_payloads)} inputs...")
        start_time = time.time()
        
        total_responses = await client.request_channel_with_callback(
            input_payloads, observer.on_response, observer.on_complete
        )
        
        end_time = time.time()
        total_time = end_time - start_time
        
        print(f"\n📊 Channel streaming completed:")
        print(f"   Input payloads sent: {len(input_payloads)}")
        print(f"   Total responses: {total_responses}")
        print(f"   Total time: {total_time:.3f} seconds")
        print(f"   Responses processed via callback: {len(observer.responses_received)}")
        
        if len(observer.responses_received) > 1:
            first_response_time = observer.responses_received[0]['timestamp']
            last_response_time = observer.responses_received[-1]['timestamp']
            processing_span = last_response_time - first_response_time
            
            print(f"   Processing span: {processing_span:.3f} seconds")
            print(f"   Average interval: {processing_span / (len(observer.responses_received) - 1):.3f} seconds")
            
        print("\n✅ Callback-based channel demonstrates true bidirectional reactivity!")
        print("   Each response was processed individually as it arrived")
        print(f"   Channel completion callback called: {observer.completed}")
        if observer.completion_error:
            print(f"   Completion error: {observer.completion_error}")
            
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()

async def main():
    """Main test function"""
    print("Waiting for server to start...")
    await asyncio.sleep(2)
    await test_callback_channel()

if __name__ == "__main__":
    asyncio.run(main())
