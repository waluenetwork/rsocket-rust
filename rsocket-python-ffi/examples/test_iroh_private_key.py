#!/usr/bin/env python3

import asyncio
import rsocket_rust

async def echo_handler(payload):
    print(f"📨 Received: {payload}")
    return payload

async def test_with_private_key():
    print("🔑 Testing Iroh transport with custom private key")
    
    test_private_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    
    iroh_transport = rsocket_rust.IrohServerTransport(test_private_key)
    
    def on_start():
        print("🎉 Iroh Server with custom private key started!")
        print(f"🔑 Using private key: {test_private_key[:16]}...")
        print("🔄 Use Ctrl+C to stop the server")
    
    server = (rsocket_rust.MultiTransportServerBuilder()
              .add_iroh_transport("Iroh-P2P", iroh_transport)
              .acceptor(echo_handler)
              .on_start(on_start))
    
    print("🔧 Server configured with custom private key")
    
    try:
        await server.serve()
    except KeyboardInterrupt:
        print("\n👋 Server stopped by user")
    except Exception as e:
        print(f"❌ Server error: {e}")

async def test_without_private_key():
    print("🔄 Testing Iroh transport with auto-generated private key")
    
    iroh_transport = rsocket_rust.IrohServerTransport()
    
    def on_start():
        print("🎉 Iroh Server with auto-generated private key started!")
        print("🔄 Use Ctrl+C to stop the server")
    
    server = (rsocket_rust.MultiTransportServerBuilder()
              .add_iroh_transport("Iroh-P2P", iroh_transport)
              .acceptor(echo_handler)
              .on_start(on_start))
    
    print("🔧 Server configured with auto-generated private key")
    
    try:
        await server.serve()
    except KeyboardInterrupt:
        print("\n👋 Server stopped by user")
    except Exception as e:
        print(f"❌ Server error: {e}")

async def main():
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "--with-key":
        await test_with_private_key()
    else:
        await test_without_private_key()

if __name__ == "__main__":
    asyncio.run(main())
