const test = require('assert');
const { 
    JsRSocketClient, 
    JsRSocketServer, 
    JsTransportConfig, 
    JsTransportType, 
    JsPayload,
    JsRSocketFactory,
    JsPerformanceMetrics,
    getSupportedTransports,
    getVersion,
    initializeLogger
} = require('../index.js');

async function testBasicFunctionality() {
    console.log('Testing basic functionality...');
    
    const version = getVersion();
    console.log('Version:', version);
    
    const transports = getSupportedTransports();
    console.log('Supported transports:', transports);
    
    initializeLogger();
    
    console.log('✅ Basic functionality test passed');
}

async function testPayloadCreation() {
    console.log('Testing payload creation...');
    
    const payload1 = JsPayload.fromString("Hello, World!", "metadata");
    test.strictEqual(payload1.getDataUtf8(), "Hello, World!");
    test.strictEqual(payload1.getMetadataUtf8(), "metadata");
    test.strictEqual(payload1.hasData(), true);
    test.strictEqual(payload1.hasMetadata(), true);
    
    const jsonData = { message: "test", value: 42 };
    const jsonMetadata = { type: "json" };
    const payload2 = JsPayload.fromJson(jsonData, jsonMetadata);
    test.strictEqual(payload2.hasData(), true);
    test.strictEqual(payload2.hasMetadata(), true);
    
    const dataBuffer = Buffer.from("binary data");
    const metadataBuffer = Buffer.from("binary metadata");
    const payload3 = new JsPayload(dataBuffer, metadataBuffer);
    test.strictEqual(payload3.dataLen(), dataBuffer.length);
    test.strictEqual(payload3.metadataLen(), metadataBuffer.length);
    
    console.log('✅ Payload creation test passed');
}

async function testTransportConfig() {
    console.log('Testing transport configuration...');
    
    const tcpConfig = new JsTransportConfig(
        JsTransportType.tcp(),
        "127.0.0.1:7878",
        null
    );
    test.strictEqual(tcpConfig.toString().includes("tcp"), true);
    
    const wsConfig = new JsTransportConfig(
        JsTransportType.websocket(),
        "ws://127.0.0.1:8080",
        null
    );
    test.strictEqual(wsConfig.toString().includes("websocket"), true);
    
    tcpConfig.setOption("test_key", "test_value");
    test.strictEqual(tcpConfig.getOption("test_key"), "test_value");
    
    tcpConfig.enableCrossbeamOptimizations();
    tcpConfig.enableSimdProcessing();
    tcpConfig.setPerformanceMode("high");
    
    const options = tcpConfig.getAllOptions();
    test.strictEqual(options["crossbeam_optimizations"], "true");
    test.strictEqual(options["simd_processing"], "true");
    test.strictEqual(options["performance_mode"], "high");
    
    console.log('✅ Transport configuration test passed');
}

async function testFactory() {
    console.log('Testing factory pattern...');
    
    const tcpClient = JsRSocketFactory.createTcpClient("127.0.0.1:7878");
    test.strictEqual(tcpClient.getTransportType(), "tcp");
    test.strictEqual(tcpClient.getAddress(), "127.0.0.1:7878");
    
    const wsClient = JsRSocketFactory.createWebsocketClient("ws://127.0.0.1:8080");
    test.strictEqual(wsClient.getTransportType(), "websocket");
    
    const quicClient = JsRSocketFactory.createQuicClient("127.0.0.1:7878");
    test.strictEqual(quicClient.getTransportType(), "quinn-quic");
    
    const optimizedClient = JsRSocketFactory.createOptimizedClient(
        "tcp",
        "127.0.0.1:7878",
        true,  // enable SIMD
        false  // disable WebWorkers
    );
    test.strictEqual(optimizedClient.getTransportType(), "tcp");
    
    const tcpServer = JsRSocketFactory.createTcpServer("127.0.0.1:7878");
    test.strictEqual(tcpServer.getTransportType(), "tcp");
    
    const wsServer = JsRSocketFactory.createWebsocketServer("127.0.0.1:8080");
    test.strictEqual(wsServer.getTransportType(), "websocket");
    
    console.log('✅ Factory pattern test passed');
}

async function testPerformanceMetrics() {
    console.log('Testing performance metrics...');
    
    const metrics = new JsPerformanceMetrics();
    
    metrics.recordRequest();
    metrics.recordRequest();
    metrics.recordResponse(10.5);
    metrics.recordResponse(15.2);
    metrics.recordError();
    metrics.recordBytesSent(1024);
    metrics.recordBytesReceived(512);
    
    test.strictEqual(metrics.getRequestCount(), 2);
    test.strictEqual(metrics.getResponseCount(), 2);
    test.strictEqual(metrics.getErrorCount(), 1);
    test.strictEqual(metrics.getBytesSent(), 1024);
    test.strictEqual(metrics.getBytesReceived(), 512);
    
    const avgLatency = metrics.getAverageLatencyMs();
    test.strictEqual(Math.abs(avgLatency - 12.85) < 0.1, true);
    
    const errorRate = metrics.getErrorRate();
    test.strictEqual(errorRate, 0.5); // 1 error out of 2 requests
    
    const bandwidth = metrics.getBandwidthMbps();
    test.strictEqual(typeof bandwidth["sent_mbps"], "number");
    test.strictEqual(typeof bandwidth["received_mbps"], "number");
    
    const summary = metrics.getSummary();
    test.strictEqual(summary["request_count"], 2);
    test.strictEqual(summary["response_count"], 2);
    test.strictEqual(summary["error_count"], 1);
    
    metrics.reset();
    test.strictEqual(metrics.getRequestCount(), 0);
    test.strictEqual(metrics.getResponseCount(), 0);
    test.strictEqual(metrics.getErrorCount(), 0);
    
    console.log('✅ Performance metrics test passed');
}

async function testClientServerLifecycle() {
    console.log('Testing client/server lifecycle...');
    
    const config = new JsTransportConfig(
        JsTransportType.tcp(),
        "127.0.0.1:7878",
        null
    );
    
    const client = new JsRSocketClient(config);
    test.strictEqual(await client.isConnected(), false);
    test.strictEqual(client.getTransportType(), "tcp");
    test.strictEqual(client.getAddress(), "127.0.0.1:7878");
    
    const server = new JsRSocketServer(config);
    test.strictEqual(await server.isRunning(), false);
    test.strictEqual(server.getTransportType(), "tcp");
    test.strictEqual(server.getAddress(), "127.0.0.1:7878");
    
    console.log('✅ Client/server lifecycle test passed');
}

async function runAllTests() {
    console.log('🧪 Running JavaScript FFI Tests...\n');
    
    try {
        await testBasicFunctionality();
        await testPayloadCreation();
        await testTransportConfig();
        await testFactory();
        await testPerformanceMetrics();
        await testClientServerLifecycle();
        
        console.log('\n🎉 All tests passed!');
        console.log('JavaScript FFI implementation is working correctly.');
        
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    }
}

if (require.main === module) {
    runAllTests();
}

module.exports = {
    runAllTests,
    testBasicFunctionality,
    testPayloadCreation,
    testTransportConfig,
    testFactory,
    testPerformanceMetrics,
    testClientServerLifecycle
};
