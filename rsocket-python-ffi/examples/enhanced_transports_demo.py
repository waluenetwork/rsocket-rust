#!/usr/bin/env python3

import asyncio
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

try:
    import rsocket_rust
    print("✅ Successfully imported rsocket_rust")
except ImportError as e:
    print(f"❌ Failed to import rsocket_rust: {e}")
    print("Please build the Python FFI first: maturin develop --features advanced-transports")
    sys.exit(1)

async def test_enhanced_transports():
    """Test P1U07 enhanced transport features"""
    print("🚀 Testing P1U07 Enhanced Transport Features")
    
    if hasattr(rsocket_rust, 'WebTransportClientTransport'):
        print("✅ WebTransport support detected")
        try:
            transport = rsocket_rust.WebTransportClientTransport("https://localhost:4433")
            print(f"  Created WebTransport client: {transport}")
        except Exception as e:
            print(f"  WebTransport client creation: {e}")
    else:
        print("⚠️  WebTransport not available (requires advanced-transports feature)")
    
    if hasattr(rsocket_rust, 'IrohRoqClientTransport'):
        print("✅ iroh-roq support detected")
        try:
            transport = rsocket_rust.IrohRoqClientTransport("iroh://endpoint")
            print(f"  Created iroh-roq client: {transport}")
        except Exception as e:
            print(f"  iroh-roq client creation: {e}")
    else:
        print("⚠️  iroh-roq not available (requires advanced-transports feature)")
    
    if hasattr(rsocket_rust, 'WebWorkersClientTransport'):
        print("✅ WebWorkers support detected")
        try:
            config = rsocket_rust.WebWorkersConfig().with_worker_count(4).with_buffer_size(1024*1024)
            transport = rsocket_rust.WebWorkersClientTransport("ws://localhost:7878", config)
            print(f"  Created WebWorkers client: {transport}")
        except Exception as e:
            print(f"  WebWorkers client creation: {e}")
    else:
        print("⚠️  WebWorkers not available (requires advanced-transports feature)")
    
    print("\n📡 Testing Standard Transport Support")
    
    transports = [
        ("TCP", lambda: rsocket_rust.TcpClientTransport("127.0.0.1:7878")),
        ("WebSocket", lambda: rsocket_rust.WebSocketClientTransport("ws://localhost:7879")),
        ("QUIC", lambda: rsocket_rust.QuinnClientTransport("127.0.0.1:7880")),
        ("Iroh P2P", lambda: rsocket_rust.IrohClientTransport("iroh://peer-id")),
    ]
    
    for name, transport_fn in transports:
        try:
            transport = transport_fn()
            print(f"✅ {name} transport created successfully")
        except Exception as e:
            print(f"❌ {name} transport failed: {e}")
    
    print("\n📦 Testing Payload Creation")
    try:
        payload = (rsocket_rust.Payload.builder()
                  .set_data_utf8("Test message")
                  .set_metadata_utf8("Test metadata")
                  .build())
        print(f"✅ Payload created: data='{payload.data_utf8()}', metadata='{payload.metadata_utf8()}'")
    except Exception as e:
        print(f"❌ Payload creation failed: {e}")
    
    print("\n🏭 Testing Factory Methods")
    factory_methods = [
        "connect_tcp",
        "connect_websocket", 
        "connect_quic",
        "connect_iroh"
    ]
    
    for method in factory_methods:
        if hasattr(rsocket_rust.RSocketFactory, method):
            print(f"✅ RSocketFactory.{method} available")
        else:
            print(f"❌ RSocketFactory.{method} not available")
    
    print("\n🎯 P1U07 Enhanced Transport Demo Complete!")
    print("All enhanced transport features have been tested.")

if __name__ == "__main__":
    asyncio.run(test_enhanced_transports())
