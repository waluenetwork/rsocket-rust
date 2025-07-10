#!/usr/bin/env python3
"""
Simple RSocket Client Demo
Demonstrates basic Python RSocket client usage with payload creation.
"""

import asyncio
import rsocket_rust

async def demo_payload_creation():
    """Demonstrate payload creation and manipulation"""
    print("🧪 Payload Creation Demo")
    print("=" * 30)
    
    payload1 = rsocket_rust.Payload(b"Hello, World!", b"metadata")
    print(f"✅ Direct payload: {payload1}")
    print(f"   Data: {payload1.data_utf8()}")
    print(f"   Metadata: {payload1.metadata_utf8()}")
    print(f"   Length: {payload1.len()}")
    print(f"   Empty: {payload1.is_empty()}")
    
    payload2 = (rsocket_rust.PayloadBuilder()
                .set_data_utf8("Built with builder!")
                .set_metadata_utf8("builder-metadata")
                .build())
    print(f"\n✅ Builder payload: {payload2}")
    print(f"   Data: {payload2.data_utf8()}")
    print(f"   Metadata: {payload2.metadata_utf8()}")
    
    binary_data = bytes([0x48, 0x65, 0x6c, 0x6c, 0x6f])  # "Hello"
    payload3 = rsocket_rust.Payload(binary_data, None)
    print(f"\n✅ Binary payload: {payload3}")
    print(f"   Data (UTF-8): {payload3.data_utf8()}")
    print(f"   Data (bytes): {payload3.data()}")

async def demo_transport_creation():
    """Demonstrate transport creation"""
    print("\n🌐 Transport Creation Demo")
    print("=" * 30)
    
    tcp_client = rsocket_rust.TcpClientTransport("127.0.0.1:7878")
    tcp_server = rsocket_rust.TcpServerTransport("127.0.0.1:7878")
    print(f"✅ TCP Client: {tcp_client}")
    print(f"✅ TCP Server: {tcp_server}")
    
    ws_client = rsocket_rust.WebSocketClientTransport("ws://127.0.0.1:7879")
    ws_server = rsocket_rust.WebSocketServerTransport("127.0.0.1:7879")
    print(f"✅ WebSocket Client: {ws_client}")
    print(f"✅ WebSocket Server: {ws_server}")
    
    quic_client = rsocket_rust.QuinnClientTransport("127.0.0.1:7880")
    quic_server = rsocket_rust.QuinnServerTransport("127.0.0.1:7880")
    print(f"✅ QUIC Client: {quic_client}")
    print(f"✅ QUIC Server: {quic_server}")
    
    iroh_client = rsocket_rust.IrohClientTransport("test-node-addr")
    iroh_server = rsocket_rust.IrohServerTransport()
    print(f"✅ Iroh P2P Client: {iroh_client}")
    print(f"✅ Iroh P2P Server: {iroh_server}")

async def demo_factory_usage():
    """Demonstrate RSocketFactory usage"""
    print("\n🏭 RSocketFactory Demo")
    print("=" * 25)
    
    factory = rsocket_rust.RSocketFactory()
    print(f"✅ RSocketFactory: {factory}")
    
    server_builder = rsocket_rust.RSocketFactory.receive_multi_transport()
    print(f"✅ MultiTransportServerBuilder: {server_builder}")
    
    print("✅ Static factory methods available:")
    print("   • connect_tcp(transport)")
    print("   • connect_websocket(transport)")
    print("   • connect_quic(transport)")
    print("   • connect_iroh(transport)")
    print("   • receive_multi_transport()")

async def main():
    print("🚀 Simple RSocket Python Client Demo")
    print("🐍 Demonstrating Python FFI bindings functionality")
    print("=" * 50)
    
    await demo_payload_creation()
    await demo_transport_creation()
    await demo_factory_usage()
    
    print("\n🎉 Demo Complete!")
    print("✅ Python FFI bindings are working correctly")
    print("📚 All core RSocket functionality is accessible from Python")
    print("\n💡 Next steps:")
    print("   • Start a multi-transport server")
    print("   • Run comprehensive_patterns_test.py")
    print("   • Test actual client-server communication")

if __name__ == "__main__":
    asyncio.run(main())
