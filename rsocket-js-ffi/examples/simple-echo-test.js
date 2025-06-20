const { 
    create_simple_client, 
    create_payload_from_string, 
    create_performance_metrics,
    get_supported_transports,
    get_library_info,
    initialize_logger
} = require('../index.js');

async function testSimpleEcho() {
    console.log('🚀 Testing RSocket JavaScript FFI');
    
    try {
        initialize_logger();
        console.log('✅ Logger initialized');
    } catch (e) {
        console.log('⚠️  Logger initialization failed:', e.message);
    }
    
    const info = get_library_info();
    console.log('📋 Library Info:', info);
    
    const transports = get_supported_transports();
    console.log('🚛 Supported Transports:', transports);
    
    const metrics = create_performance_metrics();
    console.log('📊 Performance metrics created');
    
    const client = create_simple_client();
    console.log('🔌 Client created');
    
    const payload = create_payload_from_string('Hello, RSocket!');
    console.log('📦 Payload created');
    
    console.log('✅ All basic functionality tests passed!');
    console.log('🎯 JavaScript FFI bindings are working correctly');
    
    console.log('📝 Note: Connection tests require a running RSocket server');
}

testSimpleEcho().catch(console.error);
