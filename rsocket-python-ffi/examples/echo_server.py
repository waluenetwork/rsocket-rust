#!/usr/bin/env python3
"""
Multi-Transport RSocket Echo Server Example
Demonstrates running a server that accepts connections on all transport types.
"""

import asyncio
import rsocket_rust

def echo_handler(setup_payload):
    """Simple echo handler that returns the same payload"""
    print(f"New connection with setup: {setup_payload.data_utf8()}")
    return None

async def main():
    print("🚀 Starting Multi-Transport RSocket Echo Server")
    print("📡 Supporting: TCP, WebSocket, QUIC, and Iroh P2P")
    
    tcp_transport = rsocket_rust.TcpServerTransport("127.0.0.1:7878")
    ws_transport = rsocket_rust.WebSocketServerTransport("127.0.0.1:7879")
    quic_transport = rsocket_rust.QuinnServerTransport("127.0.0.1:7880")
    iroh_transport = rsocket_rust.IrohServerTransport()
    
    def on_start():
        print("🎉 Multi-Transport Echo Server Started!")
        print("📋 Ready to accept connections")
        print("🔄 Use Ctrl+C to stop the server")
    
    server = (rsocket_rust.MultiTransportServerBuilder()
              .add_tcp_transport("TCP", tcp_transport)
              .add_websocket_transport("WebSocket", ws_transport)
              .add_quic_transport("QUIC", quic_transport)
              .add_iroh_transport("Iroh-P2P", iroh_transport)
              .acceptor(echo_handler)
              .on_start(on_start))
    
    print("🔧 Server configured with all transport types")
    
    try:
        await server.serve()
    except KeyboardInterrupt:
        print("\n👋 Server stopped by user")
    except Exception as e:
        print(f"❌ Server error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
