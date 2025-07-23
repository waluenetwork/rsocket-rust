#!/usr/bin/env python3
"""
Check if request_channel_async_generator method is properly exposed.
"""

import rsocket_rust

def check_method_exposure():
    """Check if the async generator method is exposed"""
    print("🔍 Checking Method Exposure")
    print("=" * 50)
    
    try:
        client_class = rsocket_rust.Client
        print(f"✅ Client class found: {client_class}")
        
        all_methods = [method for method in dir(client_class) if not method.startswith('_')]
        print(f"\n📋 All Client class methods ({len(all_methods)}):")
        for i, method in enumerate(all_methods, 1):
            print(f"  {i:2d}. {method}")
        
        has_async_generator = hasattr(client_class, 'request_channel_async_generator')
        print(f"\n🎯 Has 'request_channel_async_generator': {has_async_generator}")
        
        if has_async_generator:
            print("✅ Method found! Async generator method is properly exposed.")
            return True
        else:
            print("❌ Method not found in class")
            
            channel_methods = [m for m in all_methods if 'channel' in m.lower()]
            print(f"\n📡 Channel-related methods ({len(channel_methods)}):")
            for method in channel_methods:
                print(f"  • {method}")
            
            async_methods = [m for m in all_methods if 'async' in m.lower()]
            print(f"\n🔄 Async-related methods ({len(async_methods)}):")
            for method in async_methods:
                print(f"  • {method}")
            
            return False
            
    except Exception as e:
        print(f"❌ Error during testing: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = check_method_exposure()
    if success:
        print("\n🎉 Method exposure check PASSED!")
    else:
        print("\n❌ Method exposure check FAILED!")
