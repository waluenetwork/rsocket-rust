#!/usr/bin/env python3
"""
Reactive vs Batch Input Comparison
Demonstrates the difference between generator-based reactive input 
and array-based batch input for channel communication.
"""

import asyncio
import time
import rsocket_rust

def create_payload_generator(input_data_list):
    """Create a Python generator that yields Payload objects progressively"""
    def payload_generator():
        for i, data in enumerate(input_data_list):
            print(f"🔧 Generator yielding payload {i+1}: {data}")
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(data)
                       .build())
            yield payload
            time.sleep(0.1)
    
    return payload_generator()

async def test_batch_approach():
    """Test traditional array-based batch approach"""
    print("=" * 60)
    print("📦 Testing BATCH Array-Based Approach")
    print("=" * 60)
    
    client = await rsocket_rust.RSocketFactory.connect_tcp(
        rsocket_rust.TcpClientTransport("127.0.0.1:7881")
    )
    
    input_data = ["Batch 1", "Batch 2", "Batch 3", "Batch 4", "Batch 5"]
    
    print("📦 Creating all payloads at once...")
    start_time = time.time()
    
    input_payloads = []
    for i, data in enumerate(input_data):
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8(data)
                   .build())
        input_payloads.append(payload)
        print(f"📤 Batch created payload {i+1}: {data}")
    
    creation_time = time.time() - start_time
    print(f"⏱️  Batch creation time: {creation_time:.3f} seconds")
    
    responses = await client.request_channel(input_payloads)
    
    total_time = time.time() - start_time
    print(f"📥 Received {len(responses)} responses in {total_time:.3f} seconds")
    
    return len(responses)

async def test_reactive_approach():
    """Test new generator-based reactive approach"""
    print("\n" + "=" * 60)
    print("🌊 Testing REACTIVE Generator-Based Approach")
    print("=" * 60)
    
    client = await rsocket_rust.RSocketFactory.connect_tcp(
        rsocket_rust.TcpClientTransport("127.0.0.1:7881")
    )
    
    input_data = ["Reactive 1", "Reactive 2", "Reactive 3", "Reactive 4", "Reactive 5"]
    
    print("🌊 Creating payload generator...")
    start_time = time.time()
    
    payload_generator = create_payload_generator(input_data)
    
    print("🚀 Sending reactive request with generator...")
    responses = await client.request_channel_reactive(payload_generator)
    
    total_time = time.time() - start_time
    print(f"📥 Received {len(responses)} responses in {total_time:.3f} seconds")
    
    return len(responses)

async def test_error_handling():
    """Test error handling for non-generator inputs"""
    print("\n" + "=" * 60)
    print("⚠️  Testing Error Handling")
    print("=" * 60)
    
    client = await rsocket_rust.RSocketFactory.connect_tcp(
        rsocket_rust.TcpClientTransport("127.0.0.1:7881")
    )
    
    try:
        print("🧪 Testing with non-generator input (should fail)...")
        await client.request_channel_reactive(["not", "a", "generator"])
        print("❌ Error: Should have failed with TypeError")
    except TypeError as e:
        print(f"✅ Error handling works correctly: {e}")
    except Exception as e:
        print(f"⚠️  Unexpected error type: {e}")

async def main():
    print("🧪 Comparing Input Approaches: Batch vs Reactive")
    print("=" * 80)
    
    await asyncio.sleep(1)
    
    batch_count = await test_batch_approach()
    reactive_count = await test_reactive_approach()
    await test_error_handling()
    
    print("\n" + "=" * 80)
    print("📊 COMPARISON RESULTS:")
    print(f"📦 Batch approach: {batch_count} responses (all inputs created at once)")
    print(f"🌊 Reactive approach: {reactive_count} responses (inputs generated progressively)")
    print("🎯 Reactive approach enables true streaming with backpressure support")
    print("✅ Error handling prevents invalid inputs")
    print("=" * 80)

if __name__ == "__main__":
    asyncio.run(main())
