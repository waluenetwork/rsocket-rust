#!/usr/bin/env python3
"""
Debug script to check what methods are available on the RSocket client.
"""

import rsocket_rust

async def debug_client_methods():
    """Check what methods are available on the client"""
    print("🔍 Debugging RSocket Client Methods")
    print("=" * 50)
    
    try:
        print("🔗 Connecting to server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Connected successfully!")
        
        print("\n📋 Available methods on client:")
        methods = [method for method in dir(client) if not method.startswith('_')]
        for i, method in enumerate(methods, 1):
            print(f"  {i:2d}. {method}")
        
        print(f"\n🔍 Total methods: {len(methods)}")
        
        has_async_generator = hasattr(client, 'request_channel_async_generator')
        print(f"\n🎯 Has 'request_channel_async_generator': {has_async_generator}")
        
        channel_methods = [m for m in methods if 'channel' in m.lower()]
        print(f"\n📡 Channel-related methods:")
        for method in channel_methods:
            print(f"  • {method}")
            
        return client
        
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return None

if __name__ == "__main__":
    import asyncio
    asyncio.run(debug_client_methods())
