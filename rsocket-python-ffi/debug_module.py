#!/usr/bin/env python3
"""
Debug script to check Python module registration
"""

import rsocket_rust

print("=== Python Module Debug ===")
print(f"Module file: {rsocket_rust.__file__}")
print(f"Module dict keys: {list(rsocket_rust.__dict__.keys())}")

classes_to_test = [
    'Payload', 'PayloadBuilder', 'Client', 'MultiTransportServerBuilder', 'RSocketFactory',
    'TcpClientTransport', 'TcpServerTransport', 
    'WebSocketClientTransport', 'WebSocketServerTransport',
    'QuinnClientTransport', 'QuinnServerTransport',
    'IrohClientTransport', 'IrohServerTransport'
]

for class_name in classes_to_test:
    try:
        cls = getattr(rsocket_rust, class_name)
        print(f"✅ {class_name}: {cls}")
    except AttributeError as e:
        print(f"❌ {class_name}: {e}")

print("\n=== Available attributes ===")
for attr in dir(rsocket_rust):
    if not attr.startswith('__'):
        print(f"  {attr}: {getattr(rsocket_rust, attr)}")
