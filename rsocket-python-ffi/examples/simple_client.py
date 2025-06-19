#!/usr/bin/env python3
"""
Simple RSocket Client Example
Demonstrates basic client usage with request-response pattern.
"""

import asyncio
import rsocket_rust

async def main():
    print("🔗 Connecting to RSocket server via TCP...")
    
    try:
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7878")
        )
        
        print("✅ Connected successfully!")
        
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8("Hello, RSocket from Python!")
                   .set_metadata_utf8("greeting")
                   .build())
        
        print(f"📤 Sending: {payload.data_utf8()}")
        
        response = await client.request_response(payload)
        
        if response:
            print(f"📥 Received: {response.data_utf8()}")
        else:
            print("❌ No response received")
            
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
