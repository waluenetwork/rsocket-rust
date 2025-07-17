#!/usr/bin/env python3
"""
Iroh P2P RSocket Client Example
Demonstrates connecting to an Iroh echo server using node ID.
"""

import asyncio
import sys
import rsocket_rust

async def test_iroh_client(node_id):
    print(f"🔗 Connecting to Iroh echo server with NodeId: {node_id}")
    
    try:
        iroh_transport = rsocket_rust.IrohClientTransport(node_id)
        
        client = await rsocket_rust.RSocketFactory.connect_iroh(iroh_transport)
        
        print("✅ Connected to Iroh server successfully!")
        
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8("Hello from Iroh P2P client!")
                   .set_metadata_utf8("iroh-greeting")
                   .build())
        
        print(f"📤 Sending: {payload.data_utf8()}")
        
        response = await client.request_response(payload)
        
        if response:
            print(f"📥 Received echo: {response.data_utf8()}")
            print("🎉 Iroh P2P communication successful!")
        else:
            print("❌ No response received")
            
    except Exception as e:
        print(f"❌ Connection error: {e}")
        print("💡 Make sure the Iroh echo server is running and the node ID is correct")

async def main():
    if len(sys.argv) != 2:
        print("Usage: python iroh_client.py <node_id>")
        print("Example: python iroh_client.py deb401ba4856a7ec9d0a031554f7340df44e54a2c113a5b9cc8961eec361901c")
        print("\n📋 To get the node ID:")
        print("1. Start the echo server: RUST_LOG=info python examples/echo_server.py")
        print("2. Look for 'Iroh P2P server started with NodeId: <node_id>' in the logs")
        print("3. Copy the node ID and use it with this client")
        return
    
    node_id = sys.argv[1]
    await test_iroh_client(node_id)

if __name__ == "__main__":
    asyncio.run(main())
