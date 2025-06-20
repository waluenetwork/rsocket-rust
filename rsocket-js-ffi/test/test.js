const rsocket = require('../index.node');

async function testJavaScriptFFI() {
    console.log('🚀 Testing JavaScript FFI Bindings');
    
    try {
        console.log('📡 Testing TCP Client Creation');
        const tcpClient = rsocket.createClient({
            type: 'tcp',
            address: '127.0.0.1:7878'
        });
        console.log('✅ TCP client created successfully');
        
        console.log('📡 Testing WebSocket Client Creation');
        const wsClient = rsocket.createClient({
            type: 'websocket',
            address: 'ws://localhost:7879'
        });
        console.log('✅ WebSocket client created successfully');
        
        console.log('📡 Testing QUIC Client Creation');
        const quicClient = rsocket.createClient({
            type: 'quic',
            address: '127.0.0.1:7880'
        });
        console.log('✅ QUIC client created successfully');
        
        console.log('📡 Testing Iroh Client Creation');
        const irohClient = rsocket.createClient({
            type: 'iroh',
            address: 'iroh://peer-id'
        });
        console.log('✅ Iroh client created successfully');
        
        console.log('\n🎯 JavaScript FFI Test Complete!');
        console.log('All transport types created successfully.');
        
    } catch (error) {
        console.error('❌ Test failed:', error.message);
        process.exit(1);
    }
}

testJavaScriptFFI().catch(console.error);
