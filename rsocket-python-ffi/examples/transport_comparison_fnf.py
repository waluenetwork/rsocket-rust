#!/usr/bin/env python3
"""
Transport Performance Comparison
Compares performance across different transport types.
"""

import asyncio
import time
import rsocket_rust

URL = "127.0.0.1"
# URL = "3.67.9.236"
URL = "3.72.75.86"



async def benchmark_transport(transport_name, client_factory, num_requests=10000):
    """Benchmark a specific transport"""
    print(f"\n🏃 Benchmarking {transport_name} Transport ({num_requests} requests)")
    
    try:
        client = await client_factory()
        
        payload = (rsocket_rust.Payload.builder()
                   .set_data_utf8(f"Benchmark payload from {transport_name}")
                   .build())
        
        start_time = time.time()
        
        for i in range(num_requests):
            await client.fire_and_forget(payload)
            # if not response:
            #     print(f"❌ Request {i+1} failed")
            #     return None
        
        end_time = time.time()
        duration = end_time - start_time
        rps = num_requests / duration
        
        print(f"✅ {transport_name}: {duration:.3f}s total, {rps:.1f} req/s")
        return rps
        
    except Exception as e:
        print(f"❌ {transport_name} benchmark failed: {e}")
        return None



async def benchmark_concurrent(transport_name, client_factory, num_requests=10000, concurrency=100):
    """Benchmark a specific transport with controlled concurrency"""
    print(f"\n🏃 Benchmarking {transport_name} Transport "
          f"({num_requests} requests, concurrency={concurrency})")

    try:
        client = await client_factory()
        sem = asyncio.Semaphore(concurrency)

        async def send(i):
            async with sem:
                payload = (rsocket_rust.Payload.builder()
                           .set_data_utf8(f"Benchmark payload {i} from {transport_name}")
                           .build())
                try:
                    await client.fire_and_forget(payload)
                    return True
                except Exception as e:
                    print(f"❌ fire_and_forget {i} failed: {e}")
                    return False

        start_time = time.time()

        tasks = [asyncio.create_task(send(i)) for i in range(num_requests)]
        results = await asyncio.gather(*tasks)

        end_time = time.time()
        duration = end_time - start_time
        success = sum(results)
        rps = success / duration

        print(f"✅ Sent: {success}, ❌ Failed: {num_requests - success}")
        print(f"✅ {transport_name}: {duration:.3f}s total, {rps:.1f} req/s")
        return rps

    except Exception as e:
        print(f"❌ {transport_name} benchmark failed: {e}")
        return None



async def benchmark_unbounded(transport_name, client_factory, num_requests=10000):
    """Benchmark a specific transport concurrently"""
    print(f"\n🏃 Benchmarking {transport_name} Transport ({num_requests} requests, unbounded)")

    try:
        client = await client_factory()

        async def send(i):
            payload = (rsocket_rust.Payload.builder()
                       .set_data_utf8(f"Benchmark payload {i} from {transport_name}")
                       .build())
            try:
                await client.fire_and_forget(payload)
                return True
            except Exception as e:
                print(f"❌ fire_and_forget {i} failed: {e}")
                return False

        start_time = time.time()

        tasks = [asyncio.create_task(send(i)) for i in range(num_requests)]
        results = await asyncio.gather(*tasks)

        end_time = time.time()
        duration = end_time - start_time
        success = sum(results)
        rps = success / duration

        print(f"✅ Sent: {success}, ❌ Failed: {num_requests - success}")
        print(f"✅ {transport_name}: {duration:.3f}s total, {rps:.1f} req/s")
        return rps

    except Exception as e:
        print(f"❌ {transport_name} benchmark failed: {e}")
        return None


# python examples/transport_comparison_ffn.py request_response serial_loop 3.72.75.86:7878 tcp 10000
async def main():


    print("🏁 RSocket Transport Performance Comparison")
    
    print(f"URL = {URL}")

    for performance_test in ["serial_loop", "concurrent_unbounded", "concurrent_controlled"]:
        
        print(f"performance_test = {performance_test}")
    
        await asyncio.sleep(2)

        transports = [
            ("TCP", lambda: rsocket_rust.RSocketFactory.connect_tcp(
                rsocket_rust.TcpClientTransport(f"{URL}:7878"))),
            ("WebSocket", lambda: rsocket_rust.RSocketFactory.connect_websocket(
                rsocket_rust.WebSocketClientTransport(f"ws://{URL}:7879"))),
            ("QUIC", lambda: rsocket_rust.RSocketFactory.connect_quic(
                rsocket_rust.QuinnClientTransport(f"{URL}:7880"))),
        ]
        
        results = {}
        
        for transport_name, client_factory in transports:
            if performance_test == "serial_loop":
                print("serial_loop")
                rps = await benchmark_transport(transport_name, client_factory)
            elif performance_test == "concurrent_controlled":
                print("benchmark_concurrent")
                rps = await benchmark_concurrent(transport_name, client_factory)
            elif performance_test == "concurrent_unbounded":
                print("benchmark_unbounded")
                rps = await benchmark_unbounded(transport_name, client_factory)
            else:
                raise Exception("There is no test find: ", performance_test)
            
            if rps:
                results[transport_name] = rps
        
        if results:
            print("\n📊 Performance Results:")
            sorted_results = sorted(results.items(), key=lambda x: x[1], reverse=True)
            for i, (transport, rps) in enumerate(sorted_results, 1):
                print(f"  {i}. {transport}: {rps:.1f} req/s")
        else:
            print("\n❌ No successful benchmarks")

        print("\n\n------------------------------\n")

if __name__ == "__main__":
    asyncio.run(main())
