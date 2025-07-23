#!/usr/bin/env python3
"""
Test script to verify how different input types are handled by request_channel_async_generator
"""

import asyncio
import rsocket_rust
from rsocket_rust import Payload

async def test_input_types():
    """Test different input types with request_channel_async_generator"""
    print("🧪 Testing different input types with request_channel_async_generator")
    print("=" * 70)
    
    try:
        print("🔗 Connecting to server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Connected successfully!")
        
        print("\n🚀 Test 1: Regular list input...")
        list_input = [Payload(b"List item 1"), Payload(b"List item 2")]
        try:
            responses = await asyncio.wait_for(
                client.request_channel_async_generator(list_input),
                timeout=5.0
            )
            print(f"✅ List input: Received {len(responses)} responses")
        except asyncio.TimeoutError:
            print("❌ List input: Timed out (hanging)")
        except Exception as e:
            print(f"❌ List input: Error - {e}")
        
        print("\n🚀 Test 2: Regular generator input...")
        def regular_generator():
            yield Payload(b"Gen item 1")
            yield Payload(b"Gen item 2")
        
        try:
            responses = await asyncio.wait_for(
                client.request_channel_async_generator(regular_generator()),
                timeout=5.0
            )
            print(f"✅ Regular generator: Received {len(responses)} responses")
        except asyncio.TimeoutError:
            print("❌ Regular generator: Timed out (hanging)")
        except Exception as e:
            print(f"❌ Regular generator: Error - {e}")
        
        print("\n🚀 Test 3: Async generator input...")
        async def async_generator():
            yield Payload(b"Async item 1")
            await asyncio.sleep(0.01)
            yield Payload(b"Async item 2")
        
        try:
            responses = await asyncio.wait_for(
                client.request_channel_async_generator(async_generator()),
                timeout=5.0
            )
            print(f"✅ Async generator: Received {len(responses)} responses")
        except asyncio.TimeoutError:
            print("❌ Async generator: Timed out (hanging)")
        except Exception as e:
            print(f"❌ Async generator: Error - {e}")
        
        print("\n🎯 Test completed!")
        
    except Exception as e:
        print(f"❌ Connection error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(test_input_types())
