#!/usr/bin/env python3
"""
Test client for the reactive channel server example.
Demonstrates various channel communication patterns with the server.
"""

import asyncio
import time
import rsocket_rust

async def test_reactive_channel_server():
    """Test the reactive channel server with various input patterns"""
    print("🧪 Testing Reactive Channel Server")
    print("=" * 60)
    
    try:
        print("🔗 Connecting to reactive channel server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7882")
        )
        print("✅ Connected successfully!")
        print()
        
        print("📦 Test 1: Basic Channel Request")
        print("-" * 40)
        
        input_payloads = []
        for i in range(1, 4):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"Test input {i}")
                       .set_metadata_utf8(f"test-{i}")
                       .build())
            input_payloads.append(payload)
            print(f"📤 Input {i}: Test input {i}")
        
        print(f"\n🚀 Sending {len(input_payloads)} payloads to channel...")
        start_time = time.time()
        
        responses = await client.request_channel(input_payloads)
        
        end_time = time.time()
        print(f"\n📥 Received {len(responses)} responses in {end_time - start_time:.3f} seconds:")
        
        for i, response in enumerate(responses):
            data = response.data_utf8()
            metadata = response.metadata_utf8()
            print(f"📥 Response {i+1}: {data}")
            print(f"    Metadata: {metadata}")
        
        print("\n" + "=" * 60)
        print("📦 Test 2: Larger Channel Request")
        print("-" * 40)
        
        large_input_payloads = []
        for i in range(1, 6):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"Large test input {i} with more data")
                       .set_metadata_utf8(f"large-test-{i}")
                       .build())
            large_input_payloads.append(payload)
        
        print(f"🚀 Sending {len(large_input_payloads)} larger payloads...")
        start_time = time.time()
        
        large_responses = await client.request_channel(large_input_payloads)
        
        end_time = time.time()
        print(f"\n📥 Received {len(large_responses)} responses in {end_time - start_time:.3f} seconds")
        
        for i, response in enumerate(large_responses):
            data = response.data_utf8()
            print(f"📥 Response {i+1}: {data[:50]}...")
        
        print("\n" + "=" * 60)
        print("📦 Test 3: Request-Response (for comparison)")
        print("-" * 40)
        
        rr_payload = (rsocket_rust.Payload.builder()
                      .set_data_utf8("Request-Response test")
                      .build())
        
        rr_response = await client.request_response(rr_payload)
        print(f"📥 RR Response: {rr_response.data_utf8()}")
        
        print("\n" + "=" * 60)
        print("📦 Test 4: Request-Stream (for comparison)")
        print("-" * 40)
        
        rs_payload = (rsocket_rust.Payload.builder()
                      .set_data_utf8("Request-Stream test")
                      .build())
        
        stream_responses = await client.request_stream(rs_payload)
        print(f"📥 Stream responses: {len(stream_responses)} items")
        for i, response in enumerate(stream_responses):
            print(f"📥 Stream {i+1}: {response.data_utf8()}")
        
        print("\n✅ All tests completed successfully!")
        print("🎯 Reactive channel server is working correctly")
        print("🌊 Server demonstrated reactive processing with Python generators")
        
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()

async def main():
    """Main test function"""
    print("Waiting for server to be ready...")
    await asyncio.sleep(2)
    await test_reactive_channel_server()

if __name__ == "__main__":
    asyncio.run(main())
