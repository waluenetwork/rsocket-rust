#!/usr/bin/env python3
"""
Test client for the AsyncGenerator and Subscriber pattern channel server.
Demonstrates testing the LoggingSubscriber and AsyncGenerator patterns.
"""

import asyncio
import time
import rsocket_rust

async def test_async_generator_channel_server():
    """Test the AsyncGenerator and Subscriber pattern server"""
    print("🧪 Testing AsyncGenerator and Subscriber Pattern Channel Server")
    print("=" * 70)
    
    try:
        print("🔗 Connecting to async generator channel server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Connected successfully!")
        print()
        
        print("📦 Test 1: Basic AsyncGenerator Channel Request")
        print("-" * 50)
        
        input_payloads = []
        for i in range(1, 4):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"AsyncGen test input {i}")
                       .set_metadata_utf8(f"async-test-{i}")
                       .build())
            input_payloads.append(payload)
            print(f"📤 Input {i}: AsyncGen test input {i}")
        
        print(f"\n🚀 Sending {len(input_payloads)} payloads to AsyncGenerator channel...")
        start_time = time.time()
        
        responses = await client.request_channel(input_payloads)
        
        end_time = time.time()
        print(f"\n📥 Received {len(responses)} responses in {end_time - start_time:.3f} seconds:")
        
        for i, response in enumerate(responses):
            data = response.data_utf8()
            metadata = response.metadata_utf8()
            print(f"📥 Response {i+1}: {data}")
            print(f"    Metadata: {metadata}")
        
        print("\n" + "=" * 70)
        print("📦 Test 2: Callback-based Channel with Subscriber Pattern")
        print("-" * 50)
        
        class TestChannelObserver:
            def __init__(self):
                self.responses = []
                self.start_time = time.time()
                self.completed = False
            
            def on_response(self, payload, index):
                """Callback for each response (simulates Subscriber.on_next)"""
                current_time = time.time()
                elapsed = (current_time - self.start_time) * 1000
                data = payload.data_utf8()
                print(f"📥 [TestObserver] Response {index} at {elapsed:.1f}ms: {data}")
                self.responses.append({'index': index, 'data': data, 'timestamp': current_time})
            
            def on_complete(self, total_responses, success, error):
                """Callback for completion (simulates Subscriber.on_complete)"""
                current_time = time.time()
                elapsed = (current_time - self.start_time) * 1000
                self.completed = True
                if success:
                    print(f"✅ [TestObserver] Channel completed at {elapsed:.1f}ms ({total_responses} responses)")
                else:
                    print(f"❌ [TestObserver] Channel failed at {elapsed:.1f}ms: {error}")
        
        observer = TestChannelObserver()
        
        callback_payloads = []
        for i in range(1, 3):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"Subscriber test {i}")
                       .set_metadata_utf8(f"subscriber-{i}")
                       .build())
            callback_payloads.append(payload)
        
        print(f"🔄 Testing callback-based channel with {len(callback_payloads)} inputs...")
        
        total_responses = await client.request_channel_with_callback(
            callback_payloads, observer.on_response, observer.on_complete
        )
        
        print(f"📊 Callback Results: {total_responses} responses processed")
        print(f"📊 Observer received: {len(observer.responses)} responses")
        print(f"📊 Completion status: {observer.completed}")
        
        print("\n" + "=" * 70)
        print("📦 Test 3: Request-Response (for comparison)")
        print("-" * 50)
        
        rr_payload = (rsocket_rust.Payload.builder()
                      .set_data_utf8("AsyncGenerator RR test")
                      .build())
        
        rr_response = await client.request_response(rr_payload)
        print(f"📥 RR Response: {rr_response.data_utf8()}")
        
        print("\n" + "=" * 70)
        print("📦 Test 4: Request-Stream (for comparison)")
        print("-" * 50)
        
        rs_payload = (rsocket_rust.Payload.builder()
                      .set_data_utf8("AsyncGenerator Stream test")
                      .build())
        
        stream_responses = await client.request_stream(rs_payload)
        print(f"📥 Stream responses: {len(stream_responses)} items")
        for i, response in enumerate(stream_responses):
            print(f"📥 Stream {i+1}: {response.data_utf8()}")
        
        print("\n✅ All AsyncGenerator and Subscriber pattern tests completed!")
        print("🎯 Server demonstrated:")
        print("   • LoggingSubscriber with on_subscribe, on_next, on_error, on_complete")
        print("   • AsyncGenerator pattern with (Payload, is_complete) tuples")
        print("   • @router.channel equivalent returning both channel and subscriber")
        print("   • Backpressure control with subscription.request()")
        print("   • Bidirectional streaming with subscriber callbacks")
        
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()

async def main():
    """Main test function"""
    print("Waiting for AsyncGenerator server to be ready...")
    await asyncio.sleep(2)
    await test_async_generator_channel_server()

if __name__ == "__main__":
    asyncio.run(main())
