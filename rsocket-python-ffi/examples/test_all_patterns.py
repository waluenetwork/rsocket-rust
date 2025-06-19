#!/usr/bin/env python3
"""
Comprehensive RSocket Patterns Test
Tests all 4 RSocket interaction patterns across all transport types.
"""

import asyncio
import rsocket_rust

async def test_request_response(client, transport_name):
    """Test request-response pattern"""
    print(f"\n📞 Testing Request-Response on {transport_name}")
    
    payload = (rsocket_rust.Payload.builder()
               .set_data_utf8(f"Hello from {transport_name} client!")
               .build())
    
    try:
        response = await client.request_response(payload)
        if response:
            print(f"✅ {transport_name} Response: {response.data_utf8()}")
            return True
        else:
            print(f"❌ {transport_name} No response received")
            return False
    except Exception as e:
        print(f"❌ {transport_name} Request-Response failed: {e}")
        return False

async def test_fire_and_forget(client, transport_name):
    """Test fire-and-forget pattern"""
    print(f"\n🔥 Testing Fire-and-Forget on {transport_name}")
    
    payload = (rsocket_rust.Payload.builder()
               .set_data_utf8(f"Fire-and-forget from {transport_name}")
               .build())
    
    try:
        await client.fire_and_forget(payload)
        print(f"✅ {transport_name} Fire-and-forget sent successfully")
        return True
    except Exception as e:
        print(f"❌ {transport_name} Fire-and-forget failed: {e}")
        return False

async def test_request_stream(client, transport_name):
    """Test request-stream pattern"""
    print(f"\n📡 Testing Request-Stream on {transport_name}")
    
    payload = (rsocket_rust.Payload.builder()
               .set_data_utf8(f"Stream request from {transport_name}")
               .build())
    
    try:
        responses = await client.request_stream(payload)
        print(f"✅ {transport_name} Stream received {len(responses)} items")
        for i, response in enumerate(responses[:3]):
            print(f"   📦 Item {i+1}: {response.data_utf8()}")
        return True
    except Exception as e:
        print(f"❌ {transport_name} Request-stream failed: {e}")
        return False

async def test_request_channel(client, transport_name):
    """Test request-channel pattern"""
    print(f"\n🔄 Testing Request-Channel on {transport_name}")
    
    payloads = []
    for i in range(3):
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8(f"Channel message {i+1} from {transport_name}")
                   .build())
        payloads.append(payload)
    
    try:
        responses = await client.request_channel(payloads)
        print(f"✅ {transport_name} Channel received {len(responses)} responses")
        for i, response in enumerate(responses):
            print(f"   📦 Response {i+1}: {response.data_utf8()}")
        return True
    except Exception as e:
        print(f"❌ {transport_name} Request-channel failed: {e}")
        return False

async def test_transport(transport_name, client_factory):
    """Test all patterns on a specific transport"""
    print(f"\n🧪 Testing {transport_name} Transport")
    
    try:
        client = await client_factory()
        
        results = []
        results.append(await test_request_response(client, transport_name))
        results.append(await test_fire_and_forget(client, transport_name))
        results.append(await test_request_stream(client, transport_name))
        results.append(await test_request_channel(client, transport_name))
        
        success_count = sum(results)
        print(f"\n📊 {transport_name} Results: {success_count}/4 patterns successful")
        return success_count == 4
        
    except Exception as e:
        print(f"❌ {transport_name} Transport connection failed: {e}")
        return False

async def main():
    print("🧪 Comprehensive RSocket Patterns Test")
    print("🔗 Testing all 4 patterns across all transport types")
    
    await asyncio.sleep(2)
    
    transport_tests = [
        ("TCP", lambda: rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7878"))),
        ("WebSocket", lambda: rsocket_rust.RSocketFactory.connect_websocket(
            rsocket_rust.WebSocketClientTransport("ws://127.0.0.1:7879"))),
        ("QUIC", lambda: rsocket_rust.RSocketFactory.connect_quic(
            rsocket_rust.QuinnClientTransport("127.0.0.1:7880"))),
    ]
    
    all_passed = True
    for transport_name, client_factory in transport_tests:
        success = await test_transport(transport_name, client_factory)
        if not success:
            all_passed = False
    
    if all_passed:
        print("\n🎉 All transport and pattern tests passed!")
    else:
        print("\n❌ Some tests failed")

if __name__ == "__main__":
    asyncio.run(main())
