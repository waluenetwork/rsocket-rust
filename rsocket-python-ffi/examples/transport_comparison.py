#!/usr/bin/env python3
"""
Transport Performance Comparison
Compares performance across different transport types.
"""

import asyncio
import time
import rsocket_rust

async def benchmark_transport(transport_name, client_factory, num_requests=10):
    """Benchmark a specific transport"""
    print(f"\n🏃 Benchmarking {transport_name} Transport ({num_requests} requests)")
    
    try:
        client = await client_factory()
        
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8(f"Benchmark payload from {transport_name}")
                   .build())
        
        start_time = time.time()
        
        for i in range(num_requests):
            response = await client.request_response(payload)
            if not response:
                print(f"❌ Request {i+1} failed")
                return None
        
        end_time = time.time()
        duration = end_time - start_time
        rps = num_requests / duration
        
        print(f"✅ {transport_name}: {duration:.3f}s total, {rps:.1f} req/s")
        return rps
        
    except Exception as e:
        print(f"❌ {transport_name} benchmark failed: {e}")
        return None

async def main():
    print("🏁 RSocket Transport Performance Comparison")
    
    await asyncio.sleep(2)
    
    transports = [
        ("TCP", lambda: rsocket_rust.RSocketFactory.connect_tcp(
            rsocket_rust.TcpClientTransport("127.0.0.1:7878"))),
        ("WebSocket", lambda: rsocket_rust.RSocketFactory.connect_websocket(
            rsocket_rust.WebSocketClientTransport("ws://127.0.0.1:7879"))),
        ("QUIC", lambda: rsocket_rust.RSocketFactory.connect_quic(
            rsocket_rust.QuinnClientTransport("127.0.0.1:7880"))),
    ]
    
    results = {}
    
    for transport_name, client_factory in transports:
        rps = await benchmark_transport(transport_name, client_factory)
        if rps:
            results[transport_name] = rps
    
    if results:
        print("\n📊 Performance Results:")
        sorted_results = sorted(results.items(), key=lambda x: x[1], reverse=True)
        for i, (transport, rps) in enumerate(sorted_results, 1):
            print(f"  {i}. {transport}: {rps:.1f} req/s")
    else:
        print("\n❌ No successful benchmarks")

if __name__ == "__main__":
    asyncio.run(main())
