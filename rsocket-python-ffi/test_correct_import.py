#!/usr/bin/env python3
"""
Test importing from the correct module path
"""

try:
    from rsocket_rust.rsocket_rust import *
    print("✅ Successfully imported from rsocket_rust.rsocket_rust")
    
    payload = Payload(b"test data", b"test metadata")
    print(f"✅ Created Payload: {payload}")
    print(f"   Data: {payload.data_utf8()}")
    print(f"   Metadata: {payload.metadata_utf8()}")
    
    builder_payload = PayloadBuilder().set_data_utf8("Hello from builder!").build()
    print(f"✅ Created PayloadBuilder payload: {builder_payload}")
    print(f"   Data: {builder_payload.data_utf8()}")
    
    tcp_transport = TcpClientTransport("127.0.0.1:7878")
    print(f"✅ Created TcpClientTransport: {tcp_transport}")
    
    ws_transport = WebSocketClientTransport("ws://127.0.0.1:7879")
    print(f"✅ Created WebSocketClientTransport: {ws_transport}")
    
    factory = RSocketFactory()
    print(f"✅ Created RSocketFactory: {factory}")
    
    server_builder = RSocketFactory.receive_multi_transport()
    print(f"✅ Created MultiTransportServerBuilder: {server_builder}")
    
    print("\n🎉 All Python FFI bindings are working correctly!")
    
except Exception as e:
    print(f"❌ Error: {e}")
    import traceback
    traceback.print_exc()
