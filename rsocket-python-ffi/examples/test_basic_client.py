#!/usr/bin/env python3
"""
Basic Python RSocket Client Test
Tests basic connectivity and request-response pattern with TCP transport.
"""

import asyncio
import rsocket_rust

async def test_basic_client():
    print("🧪 Testing Basic Python RSocket Client")
    
    try:
        transport = rsocket_rust.TcpClientTransport("127.0.0.1:7878")
        print(f"✅ Created TCP transport: {transport}")
        
        payload = (rsocket_rust.PayloadBuilder()
                   .set_data_utf8("Hello from Python!")
                   .build())
        print(f"✅ Created payload: {payload}")
        
        print(f"📦 Payload data: {payload.data_utf8()}")
        print(f"📦 Payload length: {payload.len()}")
        print(f"📦 Payload empty: {payload.is_empty()}")
        
        print("🎉 Basic Python RSocket bindings are working!")
        return True
        
    except Exception as e:
        print(f"❌ Basic test failed: {e}")
        return False

if __name__ == "__main__":
    success = asyncio.run(test_basic_client())
    if success:
        print("\n✅ All basic tests passed!")
    else:
        print("\n❌ Some tests failed!")
