#!/usr/bin/env python3
"""
Test client for async generator reactive streaming.
Demonstrates using request_channel_async_generator with async generators for both input and output.
"""

import asyncio
import time
import rsocket_rust

class Event:
    """Simple Event implementation for completion tracking"""
    def __init__(self):
        self._is_set = False
        self._waiters = []
    
    def set(self):
        """Set the event and notify all waiters"""
        if self._is_set:
            print(f"🔄 [Event] Already set, ignoring")
            return
        print(f"🔔 [Event] Setting event, notifying {len(self._waiters)} waiters")
        self._is_set = True
        for waiter in self._waiters:
            if not waiter.done():
                waiter.set_result(None)
        self._waiters.clear()
    
    async def wait(self):
        """Wait for the event to be set"""
        if self._is_set:
            print(f"🔔 [Event] Already set, returning immediately")
            return
        
        print(f"⏳ [Event] Adding waiter, total waiters: {len(self._waiters) + 1}")
        future = asyncio.Future()
        self._waiters.append(future)
        await future
        print(f"✅ [Event] Waiter completed")

async def sample_async_publisher(wait_for_requester_complete, response_count=3):
    """
    Create an async generator that yields Payload objects.
    For use with request_channel_async_generator.
    """
    initial_payload = (rsocket_rust.Payload.builder()
                      .set_data_utf8('The quick brown fox')
                      .set_metadata_utf8('channel-route')
                      .build())
    print(f"📤 [AsyncPublisher] Sending initial payload: 'The quick brown fox'")
    yield initial_payload
    
    current_response = 0
    for i in range(response_count):
        is_complete = (current_response + 1) == response_count
        
        message = f'Item to server from client on channel: {current_response}'
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8(message)
                   .set_metadata_utf8(f"async-client-item-{current_response}")
                   .build())
        
        print(f"📤 [AsyncPublisher] Sending: {message} (complete: {is_complete})")
        yield payload
        
        if is_complete:
            print(f"🏁 [AsyncPublisher] Reached final item, setting requester completion")
            wait_for_requester_complete.set()
            break
        
        current_response += 1
        await asyncio.sleep(0.1)  # Small delay between items

async def request_channel_with_async_generators(client):
    """
    Test channel communication with async generators for both input and output.
    Demonstrates true reactive streaming with async generators.
    """
    print("🔄 Testing Async Generator Reactive Streaming")
    print("-" * 50)
    
    channel_completion_event = Event()
    requester_completion_event = Event()
    
    print("🚀 [Client] Starting async generator reactive streaming...")
    
    input_async_gen = sample_async_publisher(requester_completion_event, response_count=3)
    
    try:
        response_async_gen = await client.request_channel_async_generator(input_async_gen)
        
        print(f"📊 [Client] Async generator channel request initiated, processing responses...")
        
        response_count = 0
        received_values = []
        
        async for response_payload in response_async_gen:
            response_count += 1
            data = response_payload.data_utf8() if hasattr(response_payload, 'data_utf8') else str(response_payload)
            print(f"📥 [AsyncClient] Response {response_count}: {data}")
            
            if hasattr(response_payload, 'data') and callable(response_payload.data):
                data_list = response_payload.data()
                if isinstance(data_list, list):
                    received_values.append(bytes(data_list))
                else:
                    received_values.append(data_list)
            elif hasattr(response_payload, 'data_utf8') and callable(response_payload.data_utf8):
                received_values.append(response_payload.data_utf8().encode())
            else:
                received_values.append(str(response_payload).encode())
            
            await asyncio.sleep(0.05)
        
        print(f"🏁 [AsyncClient] Async generator completed, setting channel completion")
        channel_completion_event.set()
        
        print("⏳ [Client] Waiting for channel completion...")
        await channel_completion_event.wait()
        
        print("⏳ [Client] Waiting for requester completion...")
        await requester_completion_event.wait()
        
        expected_response_count = 8  # 2 responses per payload (4 payloads total)
        expected_patterns = [
            b'AsyncGen Response',  # All responses should contain this
            b'The quick brown fox',  # Initial payload response
            b'Item to server from client on channel: 0',  # Publisher payload 1
            b'Item to server from client on channel: 1',  # Publisher payload 2  
            b'Item to server from client on channel: 2',  # Publisher payload 3
        ]
        
        print(f"\n📊 [Results] Total responses: {response_count}")
        print(f"📊 [Results] Received values: {len(received_values)}")
        
        print("\n📋 [Validation] Checking response values...")
        for i, value in enumerate(received_values):
            print(f"📋 [Validation] Value {i}: {value}")
        
        if len(received_values) == expected_response_count:
            print("✅ [Validation] Received expected number of responses!")
            
            all_responses_text = b' '.join(received_values).decode('utf-8', errors='ignore')
            patterns_found = []
            for pattern in expected_patterns:
                if pattern.decode('utf-8') in all_responses_text:
                    patterns_found.append(pattern)
                    print(f"✅ [Validation] Found expected pattern: {pattern}")
                else:
                    print(f"❌ [Validation] Missing expected pattern: {pattern}")
            
            if len(patterns_found) == len(expected_patterns):
                print("✅ [Validation] All expected patterns found in responses!")
                return True
            else:
                print(f"❌ [Validation] Only {len(patterns_found)}/{len(expected_patterns)} patterns found")
                return False
        else:
            print(f"❌ [Validation] Expected {expected_response_count} responses, got {len(received_values)}")
            return False
            
    except Exception as e:
        print(f"❌ [Error] Async generator channel communication failed: {e}")
        import traceback
        traceback.print_exc()
        return False

async def test_async_generator_reactive_streaming():
    """Test the async generator reactive streaming implementation"""
    print("🧪 Testing Async Generator Reactive Streaming")
    print("🎯 Using async generators for both input and output")
    print("=" * 80)
    
    try:
        print("🔗 Connecting to async generator channel server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Connected successfully!")
        print()
        
        success = await request_channel_with_async_generators(client)
        
        if success:
            print("\n🎉 Async generator reactive streaming test PASSED!")
        else:
            print("\n❌ Async generator reactive streaming test FAILED!")
        
        print("\n✅ All async generator reactive streaming tests completed!")
        print("🎯 Server demonstrated:")
        print("   • Async generator input processing with __anext__() iteration")
        print("   • Async generator output yielding responses as they arrive")
        print("   • True reactive streaming on both input and output sides")
        print("   • Event-based completion tracking maintained")
        print("   • No callback dependencies - pure async generator pattern")
        print("   • Response validation matching expected pattern")
        
        return success
        
    except Exception as e:
        print(f"❌ Test error: {e}")
        import traceback
        traceback.print_exc()
        return False

async def main():
    """Main test function"""
    print("Waiting for AsyncGenerator server to be ready...")
    await asyncio.sleep(2)
    
    success = await test_async_generator_reactive_streaming()
    
    if success:
        print("\n🎉 All async generator tests PASSED! True reactive streaming working correctly.")
    else:
        print("\n❌ Some async generator tests FAILED! Check the implementation.")

if __name__ == "__main__":
    asyncio.run(main())
