#!/usr/bin/env python3
"""
Test top-level imports with __init__.py re-exports
"""

try:
    import rsocket_rust
    print('✅ Top-level import works!')
    
    payload = rsocket_rust.Payload(b'test', b'meta')
    print(f'✅ Payload: {payload}')
    
    builder = rsocket_rust.PayloadBuilder().set_data_utf8('Hello!').build()
    print(f'✅ PayloadBuilder: {builder}')
    
    tcp = rsocket_rust.TcpClientTransport('127.0.0.1:7878')
    print(f'✅ TcpClientTransport: {tcp}')
    
    ws = rsocket_rust.WebSocketClientTransport('ws://127.0.0.1:7879')
    print(f'✅ WebSocketClientTransport: {ws}')
    
    quic = rsocket_rust.QuinnClientTransport('127.0.0.1:7880')
    print(f'✅ QuinnClientTransport: {quic}')
    
    iroh = rsocket_rust.IrohClientTransport('test-node-addr')
    print(f'✅ IrohClientTransport: {iroh}')
    
    factory = rsocket_rust.RSocketFactory()
    print(f'✅ RSocketFactory: {factory}')
    
    server_builder = rsocket_rust.RSocketFactory.receive_multi_transport()
    print(f'✅ MultiTransportServerBuilder: {server_builder}')
    
    print('🎉 All top-level imports working perfectly!')
    
except Exception as e:
    print(f'❌ Error: {e}')
    import traceback
    traceback.print_exc()
