#!/usr/bin/env python3
"""
Test script for the new request_channel_async_generator method.
"""

import asyncio
import rsocket_rust
from rsocket_rust import Payload
import logging

logging.basicConfig(level=logging.INFO)

async def test_async_generator():
    """Test the async generator method"""
    print("🧪 Testing request_channel_async_generator method")
    print("=" * 60)
    
    try:
        print("🔗 Connecting to server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Connected successfully!")
        
        print("\n🚀 Testing with regular list input...")
        test_payloads = [
            Payload(b"Test message 1"),
            Payload(b"Test message 2"),
            Payload(b"Test message 3")
        ]
        
        responses = await client.request_channel_async_generator(test_payloads)
        
        print(f"\n📥 Received {len(responses)} responses:")
        for i, response in enumerate(responses, 1):
            data = response.data_utf8()
            if data:
                print(f"  {i}. {data}")
            else:
                print(f"  {i}. [No data]")
        
        print("\n✅ Async generator method test completed successfully!")
        return True
        
    except Exception as e:
        print(f"❌ Error during test: {e}")
        import traceback
        traceback.print_exc()
        return False

async def test_with_async_generator():
    """Test with actual async generator (more complex case)"""
    print("\n🧪 Testing with actual async generator")
    print("=" * 60)
    
    try:
        print("🔗 Connecting to server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Connected successfully!")
        
        async def sample_async_generator():
            """Sample async generator for testing"""
            for i in range(3):
                message = f"Async gen item {i}"
                print(f"📤 [AsyncGen] Yielding: {message}")
                yield Payload(message.encode('utf-8'))
                await asyncio.sleep(0.1)
        
        print("\n🚀 Testing request_channel_async_generator with async generator...")
        
        generator = sample_async_generator()
        responses = await client.request_channel_async_generator(generator)
        
        print(f"\n📥 Received {len(responses)} responses:")
        for i, response in enumerate(responses, 1):
            data = response.data_utf8()
            if data:
                print(f"  {i}. {data}")
            else:
                print(f"  {i}. [No data]")
        
        print("\n✅ Async generator test completed successfully!")
        return True
        
    except Exception as e:
        print(f"❌ Error during async generator test: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success1 = asyncio.run(test_async_generator())
    success2 = asyncio.run(test_with_async_generator())
    
    if success1 and success2:
        print("\n🎉 All tests PASSED!")
    else:
        print("\n❌ Some tests FAILED!")
