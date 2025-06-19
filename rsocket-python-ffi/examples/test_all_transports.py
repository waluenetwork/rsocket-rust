#!/usr/bin/env python3
"""
Comprehensive Transport Test
Tests all transport types: TCP, WebSocket, QUIC, and Iroh P2P.
"""

import asyncio
import rsocket_rust

async def test_transport_creation():
    """Test creating all transport types"""
    print("🧪 Testing All Transport Types")
    
    try:
        tcp_client = rsocket_rust.TcpClientTransport("127.0.0.1:7878")
        tcp_server = rsocket_rust.TcpServerTransport("127.0.0.1:7878")
        print("✅ TCP transports created successfully")
        
        ws_client = rsocket_rust.WebSocketClientTransport("ws://127.0.0.1:7879")
        ws_server = rsocket_rust.WebSocketServerTransport("127.0.0.1:7879")
        print("✅ WebSocket transports created successfully")
        
        quic_client = rsocket_rust.QuinnClientTransport("127.0.0.1:7880")
        quic_server = rsocket_rust.QuinnServerTransport("127.0.0.1:7880")
        print("✅ QUIC transports created successfully")
        
        iroh_client = rsocket_rust.IrohClientTransport("test-node-addr")
        iroh_server = rsocket_rust.IrohServerTransport()
        print("✅ Iroh P2P transports created successfully")
        
        return True
        
    except Exception as e:
        print(f"❌ Transport creation failed: {e}")
        return False

async def test_factory_methods():
    """Test RSocketFactory static methods"""
    print("\n🏭 Testing RSocketFactory Methods")
    
    try:
        factory = rsocket_rust.RSocketFactory()
        print(f"✅ Created RSocketFactory: {factory}")
        
        server_builder = rsocket_rust.RSocketFactory.receive_multi_transport()
        print(f"✅ Created MultiTransportServerBuilder: {server_builder}")
        
        return True
        
    except Exception as e:
        print(f"❌ Factory methods failed: {e}")
        return False

async def main():
    print("🚀 Comprehensive Transport Test")
    
    results = []
    results.append(await test_transport_creation())
    results.append(await test_factory_methods())
    
    if all(results):
        print("\n🎉 All transport tests passed!")
    else:
        print("\n❌ Some transport tests failed!")

if __name__ == "__main__":
    asyncio.run(main())
