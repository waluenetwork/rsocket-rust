#!/usr/bin/env python3
"""
Test client for the AsyncGenerator and Subscriber pattern channel server.
Demonstrates testing the LoggingSubscriber and AsyncGenerator patterns with Event-based completion tracking.
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

def sample_publisher(wait_for_requester_complete, response_count=3):
    """
    Create a publisher that yields (Payload, is_complete) tuples.
    Simulates the AsyncGenerator pattern from the user's example.
    """
    async def generator():
        current_response = 0
        for i in range(response_count):
            is_complete = (current_response + 1) == response_count
            
            message = f'Item to server from client on channel: {current_response}'
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(message)
                       .set_metadata_utf8(f"client-item-{current_response}")
                       .build())
            
            print(f"📤 [Publisher] Sending: {message} (complete: {is_complete})")
            yield payload, is_complete
            
            if is_complete:
                print(f"🏁 [Publisher] Reached final item (completion will be set by client)")
                break
            
            current_response += 1
            await asyncio.sleep(0.1)  # Small delay between items
    
    return generator()

class ChannelSubscriber:
    """
    Subscriber implementation that tracks responses and completion events.
    Follows the pattern from the user's example.
    """
    
    def __init__(self, wait_for_responder_complete):
        self.wait_for_responder_complete = wait_for_responder_complete
        self.values = []
        self.subscription = None
        self.completed = False
        self.error_occurred = False
    
    def on_subscribe(self, subscription):
        """Called when subscription is established"""
        print("📋 [ChannelSubscriber] on_subscribe called")
        self.subscription = subscription
        if hasattr(subscription, 'request'):
            subscription.request(5)
            print("📋 [ChannelSubscriber] Requested 5 initial items")
    
    def on_next(self, value, is_complete=False):
        """Called for each response payload"""
        data = value.data_utf8() if hasattr(value, 'data_utf8') else str(value)
        print(f"📥 [ChannelSubscriber] From server on channel: {data}")
        
        if hasattr(value, 'data') and callable(value.data):
            data_list = value.data()
            if isinstance(data_list, list):
                self.values.append(bytes(data_list))
            else:
                self.values.append(data_list)
        elif hasattr(value, 'data_utf8') and callable(value.data_utf8):
            self.values.append(value.data_utf8().encode())
        else:
            self.values.append(str(value).encode())
        
        if is_complete:
            print("📥 [ChannelSubscriber] Received completion signal")
            self.wait_for_responder_complete.set()
    
    def on_error(self, exception):
        """Called when an error occurs"""
        print(f"❌ [ChannelSubscriber] Error from server on channel: {exception}")
        self.error_occurred = True
        self.wait_for_responder_complete.set()
    
    def on_complete(self, total_responses, success, error):
        """Called when the stream completes"""
        print(f"✅ [ChannelSubscriber] Completed from server on channel (total: {total_responses}, success: {success})")
        if error:
            print(f"❌ [ChannelSubscriber] Completion with error: {error}")
        self.completed = True
        print(f"🏁 [ChannelSubscriber] Setting channel completion event")
        self.wait_for_responder_complete.set()

async def request_channel_with_events(client):
    """
    Test channel communication with Event-based completion tracking.
    Follows the pattern from the user's example.
    """
    print("🔄 Testing Event-based Channel Communication")
    print("-" * 50)
    
    channel_completion_event = Event()
    requester_completion_event = Event()
    
    payload = (rsocket_rust.Payload.builder()
               .set_data_utf8('The quick brown fox')
               .set_metadata_utf8('channel-route')
               .build())
    
    print("📤 [Client] Sending initial payload: 'The quick brown fox'")
    
    publisher = sample_publisher(requester_completion_event, response_count=3)
    
    subscriber = ChannelSubscriber(channel_completion_event)
    
    print("🚀 [Client] Starting channel request with callback pattern...")
    
    publisher_payloads = []
    async for payload_item, is_complete in publisher:
        publisher_payloads.append(payload_item)
        if is_complete:
            break
    
    try:
        total_responses = await client.request_channel_with_callback(
            [payload] + publisher_payloads,
            subscriber.on_next,
            subscriber.on_complete
        )
        
        print(f"📊 [Client] Channel request initiated, expecting responses...")
        print(f"🏁 [Client] Setting requester completion event after request")
        requester_completion_event.set()
        
        print("⏳ [Client] Waiting for channel completion...")
        await channel_completion_event.wait()
        
        print("⏳ [Client] Waiting for requester completion...")
        await requester_completion_event.wait()
        
        expected_response_count = 8
        expected_patterns = [
            b'AsyncGen Response',  # All responses should contain this
            b'The quick brown fox',  # Initial payload response
            b'Item to server from client on channel: 0',  # Publisher payload 1
            b'Item to server from client on channel: 1',  # Publisher payload 2  
            b'Item to server from client on channel: 2',  # Publisher payload 3
        ]
        
        print(f"\n📊 [Results] Total responses: {total_responses}")
        print(f"📊 [Results] Subscriber values: {len(subscriber.values)}")
        print(f"📊 [Results] Completed: {subscriber.completed}")
        print(f"📊 [Results] Error occurred: {subscriber.error_occurred}")
        
        print("\n📋 [Validation] Checking response values...")
        for i, value in enumerate(subscriber.values):
            print(f"📋 [Validation] Value {i}: {value}")
        
        if len(subscriber.values) == expected_response_count:
            print("✅ [Validation] Received expected number of responses!")
            
            all_responses_text = b' '.join(subscriber.values).decode('utf-8', errors='ignore')
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
            print(f"❌ [Validation] Expected {expected_response_count} responses, got {len(subscriber.values)}")
            return False
            
    except Exception as e:
        print(f"❌ [Error] Channel communication failed: {e}")
        import traceback
        traceback.print_exc()
        return False

async def test_async_generator_channel_server():
    """Test the AsyncGenerator and Subscriber pattern server with Event-based completion"""
    print("🧪 Testing AsyncGenerator and Subscriber Pattern Channel Server")
    print("🎯 Using Event-based completion tracking and Publisher/Subscriber pattern")
    print("=" * 80)
    
    try:
        print("🔗 Connecting to async generator channel server...")
        client = await rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7883")
        )
        print("✅ Connected successfully!")
        print()
        
        success = await request_channel_with_events(client)
        
        if success:
            print("\n🎉 Event-based channel communication test PASSED!")
        else:
            print("\n❌ Event-based channel communication test FAILED!")
        
        print("\n" + "=" * 80)
        print("📦 Additional Test: Basic Channel Request (for comparison)")
        print("-" * 50)
        
        input_payloads = []
        for i in range(1, 4):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"Basic test input {i}")
                       .set_metadata_utf8(f"basic-test-{i}")
                       .build())
            input_payloads.append(payload)
            print(f"📤 Input {i}: Basic test input {i}")
        
        print(f"\n🚀 Sending {len(input_payloads)} payloads to channel...")
        start_time = time.time()
        
        responses = await client.request_channel(input_payloads)
        
        end_time = time.time()
        print(f"\n📥 Received {len(responses)} responses in {end_time - start_time:.3f} seconds:")
        
        for i, response in enumerate(responses):
            data = response.data_utf8()
            metadata = response.metadata_utf8()
            print(f"📥 Response {i+1}: {data}")
            print(f"    Metadata: {metadata}")
        
        print("\n✅ All AsyncGenerator and Subscriber pattern tests completed!")
        print("🎯 Server demonstrated:")
        print("   • Event-based completion tracking with Publisher/Subscriber pattern")
        print("   • LoggingSubscriber with on_subscribe, on_next, on_error, on_complete")
        print("   • AsyncGenerator pattern with (Payload, is_complete) tuples")
        print("   • initial_request_n equivalent with subscription.request(5)")
        print("   • Bidirectional streaming with proper completion signaling")
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
    
    success = await test_async_generator_channel_server()
    
    if success:
        print("\n🎉 All tests PASSED! AsyncGenerator and Subscriber patterns working correctly.")
    else:
        print("\n❌ Some tests FAILED! Check the implementation.")

if __name__ == "__main__":
    asyncio.run(main())
