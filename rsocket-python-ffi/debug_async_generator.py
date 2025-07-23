#!/usr/bin/env python3
"""
Debug script for request_channel_async_generator method.
Simple test to isolate hanging issue.
"""

import asyncio
import rsocket_rust
from rsocket_rust import Payload
import time

async def debug_async_generator():
    """Debug the async generator method with minimal test case"""
    print("🔍 Debug: Testing request_channel_async_generator method")
    print("=" * 60)
    
    try:
        print("🔗 Debug: Connecting to server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Debug: Connected successfully!")
        
        print("🚀 Debug: Creating simple test payload...")
        test_payloads = [
            Payload(b"Debug message 1")
        ]
        print(f"✅ Debug: Created {len(test_payloads)} test payloads")
        
        print("📤 Debug: About to call request_channel_async_generator...")
        start_time = time.time()
        
        try:
            responses = await asyncio.wait_for(
                client.request_channel_async_generator(test_payloads),
                timeout=10.0
            )
            end_time = time.time()
            print(f"✅ Debug: Method completed in {end_time - start_time:.2f} seconds")
            print(f"📥 Debug: Received {len(responses)} responses")
            
            for i, response in enumerate(responses, 1):
                print(f"  {i}. {response}")
                
        except asyncio.TimeoutError:
            print("❌ Debug: Method timed out after 10 seconds - likely hanging!")
            return False
        
        print("✅ Debug: Test completed successfully!")
        return True
        
    except Exception as e:
        print(f"❌ Debug: Error during test: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = asyncio.run(debug_async_generator())
    if success:
        print("\n🎉 Debug test PASSED!")
    else:
        print("\n❌ Debug test FAILED!")
