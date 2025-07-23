#!/usr/bin/env python3
"""
Test script to debug method exposure issues.
"""

import rsocket_rust
import inspect

def test_method_exposure():
    """Test if request_channel_async_generator is properly exposed"""
    print("🔍 Testing Method Exposure")
    print("=" * 50)
    
    try:
        print(f"✅ rsocket_rust module imported successfully")
        print(f"📍 Module location: {rsocket_rust.__file__}")
        
        client_class = rsocket_rust.Client
        print(f"✅ Client class found: {client_class}")
        
        all_methods = [method for method in dir(client_class) if not method.startswith('_')]
        print(f"\n📋 All Client class methods ({len(all_methods)}):")
        for i, method in enumerate(all_methods, 1):
            print(f"  {i:2d}. {method}")
        
        has_async_generator = hasattr(client_class, 'request_channel_async_generator')
        print(f"\n🎯 Has 'request_channel_async_generator': {has_async_generator}")
        
        if has_async_generator:
            method = getattr(client_class, 'request_channel_async_generator')
            print(f"✅ Method found: {method}")
            print(f"📝 Method signature: {inspect.signature(method)}")
        else:
            print("❌ Method not found in class")
            
        channel_methods = [m for m in all_methods if 'channel' in m.lower()]
        print(f"\n📡 Channel-related methods ({len(channel_methods)}):")
        for method in channel_methods:
            print(f"  • {method}")
            
        if 'request_channel_reactive_streaming' in all_methods:
            reactive_method = getattr(client_class, 'request_channel_reactive_streaming')
            print(f"\n🔍 Comparison method 'request_channel_reactive_streaming':")
            print(f"  Type: {type(reactive_method)}")
            try:
                print(f"  Signature: {inspect.signature(reactive_method)}")
            except Exception as e:
                print(f"  Signature error: {e}")
        
        return has_async_generator
        
    except Exception as e:
        print(f"❌ Error during testing: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    test_method_exposure()
