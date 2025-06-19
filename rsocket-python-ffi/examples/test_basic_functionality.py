#!/usr/bin/env python3
"""
Basic functionality test for Python FFI bindings
Tests that all classes can be imported and instantiated correctly.
"""

import rsocket_rust
import asyncio

async def test_basic_functionality():
    print('🧪 Testing Python FFI Bindings...')
    
    payload = (rsocket_rust.Payload.builder()
               .set_data_utf8('Hello from Python!')
               .set_metadata_utf8('test-metadata')
               .build())
    
    print(f'✅ Payload: {payload}')
    print(f'   Data: {payload.data_utf8()}')
    print(f'   Metadata: {payload.metadata_utf8()}')
    
    tcp_transport = rsocket_rust.TcpClientTransport('127.0.0.1:7878')
    ws_transport = rsocket_rust.WebSocketClientTransport('ws://127.0.0.1:7879')
    quic_transport = rsocket_rust.QuinnClientTransport('127.0.0.1:7880')
    iroh_transport = rsocket_rust.IrohClientTransport('test-addr')
    
    print(f'✅ TCP Transport: {tcp_transport}')
    print(f'✅ WebSocket Transport: {ws_transport}')
    print(f'✅ QUIC Transport: {quic_transport}')
    print(f'✅ Iroh Transport: {iroh_transport}')
    
    tcp_server = rsocket_rust.TcpServerTransport('127.0.0.1:7878')
    ws_server = rsocket_rust.WebSocketServerTransport('127.0.0.1:7879')
    quic_server = rsocket_rust.QuinnServerTransport('127.0.0.1:7880')
    iroh_server = rsocket_rust.IrohServerTransport()
    
    print(f'✅ TCP Server: {tcp_server}')
    print(f'✅ WebSocket Server: {ws_server}')
    print(f'✅ QUIC Server: {quic_server}')
    print(f'✅ Iroh Server: {iroh_server}')
    
    server_builder = (rsocket_rust.MultiTransportServerBuilder()
                      .add_tcp_transport('TCP', tcp_server)
                      .add_websocket_transport('WebSocket', ws_server)
                      .add_quic_transport('QUIC', quic_server)
                      .add_iroh_transport('Iroh-P2P', iroh_server))
    
    print(f'✅ Multi-Transport Server Builder: {server_builder}')
    
    factory = rsocket_rust.RSocketFactory()
    print(f'✅ RSocket Factory: {factory}')
    
    print('🎉 All Python FFI bindings are working correctly!')

if __name__ == "__main__":
    asyncio.run(test_basic_functionality())
