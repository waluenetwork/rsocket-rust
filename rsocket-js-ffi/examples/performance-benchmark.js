const { 
    create_simple_client, 
    create_payload_from_string, 
    create_performance_metrics
} = require('../index.js');

async function performanceBenchmark() {
    console.log('🏃‍♂️ RSocket JavaScript FFI Performance Benchmark');
    console.log('================================================');
    
    const metrics = create_performance_metrics();
    const iterations = 10000;
    
    console.log(`\n📦 Testing payload creation (${iterations} iterations)...`);
    const payloadStart = Date.now();
    
    for (let i = 0; i < iterations; i++) {
        const payload = create_payload_from_string(`Test message ${i}`);
        metrics.record_request();
    }
    
    const payloadDuration = Date.now() - payloadStart;
    console.log(`✅ Payload creation: ${iterations} operations in ${payloadDuration}ms`);
    console.log(`   Average: ${(payloadDuration / iterations).toFixed(3)}ms per operation`);
    console.log(`   Throughput: ${(iterations / (payloadDuration / 1000)).toFixed(0)} ops/sec`);
    
    console.log(`\n🔌 Testing client creation (1000 iterations)...`);
    const clientStart = Date.now();
    
    for (let i = 0; i < 1000; i++) {
        const client = create_simple_client();
    }
    
    const clientDuration = Date.now() - clientStart;
    console.log(`✅ Client creation: 1000 operations in ${clientDuration}ms`);
    console.log(`   Average: ${(clientDuration / 1000).toFixed(3)}ms per operation`);
    
    console.log('\n📊 Performance Metrics Summary:');
    console.log(`   Request Count: ${metrics.get_request_count()}`);
    console.log(`   Uptime: ${metrics.get_uptime_seconds().toFixed(2)} seconds`);
    
    console.log('\n🎯 JavaScript FFI Performance Test Complete!');
}

performanceBenchmark().catch(console.error);
