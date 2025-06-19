#!/usr/bin/env python3
"""
Minimal test to check if any classes are available
"""

try:
    import rsocket_rust
    print("✅ Module imported successfully")
    print(f"Module: {rsocket_rust}")
    print(f"Module file: {getattr(rsocket_rust, '__file__', 'None')}")
    print(f"Module path: {getattr(rsocket_rust, '__path__', 'None')}")
    
    print(f"Is package: {hasattr(rsocket_rust, '__path__')}")
    
    if hasattr(rsocket_rust, '__path__'):
        print("This appears to be a namespace package, not the compiled module")
        try:
            from rsocket_rust import rsocket_rust as actual_module
            print(f"Found actual module: {actual_module}")
        except ImportError as e:
            print(f"Could not import actual module: {e}")
    
except ImportError as e:
    print(f"❌ Failed to import module: {e}")
    import traceback
    traceback.print_exc()
