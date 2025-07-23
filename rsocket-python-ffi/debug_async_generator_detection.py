#!/usr/bin/env python3
"""
Debug script to test async generator detection in request_channel_async_generator
"""

import asyncio
import rsocket_rust
from rsocket_rust import Payload

async def debug_detection():
    """Debug which code path async generators take"""
    print("🔍 Debug: Testing async generator detection")
    print("=" * 60)
    
    try:
        print("🔗 Debug: Connecting to server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Debug: Connected successfully!")
        
        print("\n🚀 Test 1: Regular list")
        list_input = [Payload(b"List item")]
        try:
            responses = await asyncio.wait_for(
                client.request_channel_async_generator(list_input),
                timeout=3.0
            )
            print(f"✅ List: Received {len(responses)} responses")
        except Exception as e:
            print(f"❌ List: Error - {e}")
        
        print("\n🚀 Test 2: Regular generator")
        def regular_gen():
            yield Payload(b"Gen item")
        
        try:
            responses = await asyncio.wait_for(
                client.request_channel_async_generator(regular_gen()),
                timeout=3.0
            )
            print(f"✅ Regular gen: Received {len(responses)} responses")
        except Exception as e:
            print(f"❌ Regular gen: Error - {e}")
        
        print("\n🚀 Test 3: Async generator")
        async def async_gen():
            yield Payload(b"Async item")
        
        ag = async_gen()
        print(f"Debug: Async generator type: {type(ag)}")
        print(f"Debug: Has __anext__: {hasattr(ag, '__anext__')}")
        print(f"Debug: Has __next__: {hasattr(ag, '__next__')}")
        print(f"Debug: Has __aiter__: {hasattr(ag, '__aiter__')}")
        print(f"Debug: Has __iter__: {hasattr(ag, '__iter__')}")
        
        try:
            responses = await asyncio.wait_for(
                client.request_channel_async_generator(ag),
                timeout=3.0
            )
            print(f"✅ Async gen: Received {len(responses)} responses")
        except asyncio.TimeoutError:
            print("❌ Async gen: Timed out (still hanging)")
        except Exception as e:
            print(f"❌ Async gen: Error - {e}")
            print(f"    Error type: {type(e)}")
        
        print("\n🎯 Detection test completed!")
        
    except Exception as e:
        print(f"❌ Connection error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(debug_detection())
