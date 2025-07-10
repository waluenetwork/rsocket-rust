#!/usr/bin/env python3
"""
Multi-Transport Server Test
Tests the multi-transport server builder functionality.
"""

import asyncio
import rsocket_rust

def echo_handler(setup_payload):
    """Simple echo handler"""
    print(f"📞 New connection with setup: {setup_payload}")
    return None  # Use default EchoRSocket

def on_start_handler():
    """Server start handler"""
    print("🎉 Multi-Transport Server Started!")

async def test_multi_transport_server():
    """Test multi-transport server configuration"""
    print("🧪 Testing Multi-Transport Server")
    
    try:
        tcp_transport = rsocket_rust.TcpServerTransport("127.0.0.1:7878")
        ws_transport = rsocket_rust.WebSocketServerTransport("127.0.0.1:7879")
        quic_transport = rsocket_rust.QuinnServerTransport("127.0.0.1:7880")
        iroh_transport = rsocket_rust.IrohServerTransport()
        
        print("✅ Created all server transports")
        
        server_builder = (rsocket_rust.RSocketFactory.receive_multi_transport()
                         .add_tcp_transport("TCP", tcp_transport)
                         .add_websocket_transport("WebSocket", ws_transport)
                         .add_quic_transport("QUIC", quic_transport)
                         .add_iroh_transport("Iroh-P2P", iroh_transport)
                         .acceptor(echo_handler)
                         .fragment(1024)
                         .on_start(on_start_handler))
        
        print("✅ Multi-transport server configured successfully")
        print("📋 Server supports: TCP, WebSocket, QUIC, and Iroh P2P")
        
        print("⚠️  Server configured but not started (would block)")
        
        return True
        
    except Exception as e:
        print(f"❌ Multi-transport server test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

async def main():
    print("🚀 Multi-Transport Server Test")
    
    success = await test_multi_transport_server()
    
    if success:
        print("\n🎉 Multi-transport server test passed!")
    else:
        print("\n❌ Multi-transport server test failed!")

if __name__ == "__main__":
    asyncio.run(main())
