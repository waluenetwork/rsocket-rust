#!/usr/bin/env python3
"""
Multi-Transport RSocket Echo Server Example
Demonstrates running a server that accepts connections on all transport types.
"""

import asyncio
import rsocket_rust

def custom_request_response(payload):
    """Handle request-response requests"""
    data = payload.data_utf8() if payload.data_utf8() else "No data"
    print(f"📞 Request-Response: {data}")
    
    response = (rsocket_rust.Payload.builder()
                .set_data_utf8(f"Custom Echo: {data}")
                .set_metadata_utf8("custom-response")
                .build())
    return response


async def main():
    print("🚀 Starting Multi-Transport RSocket Echo Server")
    print("📡 Supporting: TCP, WebSocket, QUIC, and Iroh P2P")
    
    handler = (rsocket_rust.RSocketHandler()
            .request_response(custom_request_response))
    
    tcp_transport = rsocket_rust.TcpServerTransport("0.0.0.0:7878")
    ws_transport  = rsocket_rust.WebSocketServerTransport("0.0.0.0:7879")
    quic_transport = rsocket_rust.QuinnServerTransport("0.0.0.0:7880")
    iroh_transport = rsocket_rust.IrohServerTransport()
    
    def on_start():
        print("🎉 Multi-Transport Echo Server Started!")
        print("📋 Ready to accept connections")
        print("🔗 Iroh P2P transport is running")
        print("💡 Node ID will be available once a client connects or through logs")
        print("🔄 Use Ctrl+C to stop the server")
    
    server = (rsocket_rust.MultiTransportServerBuilder()
              .add_tcp_transport("TCP", tcp_transport)
              .add_websocket_transport("WebSocket", ws_transport)
              .add_quic_transport("QUIC", quic_transport)
              .add_iroh_transport("Iroh-P2P", iroh_transport)
              .acceptor(handler)
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
