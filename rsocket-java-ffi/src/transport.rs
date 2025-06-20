use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum JavaRSocketTransportType {
    TCP = 0,
    WEBSOCKET = 1,
    QUIC = 2,
    IROH_P2P = 3,
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Transport_getSupportedTransports(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let transports = "tcp,websocket";
    let output = env.new_string(transports).expect("Couldn't create java string!");
    output.into_inner()
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Transport_parseTransportType(
    env: JNIEnv,
    _class: JClass,
    transport_str: JString,
) -> jint {
    let transport = match env.get_string(transport_str) {
        Ok(s) => s.into(),
        Err(_) => return -1,
    };
    
    match transport.as_str() {
        "tcp" => JavaRSocketTransportType::TCP as jint,
        "websocket" => JavaRSocketTransportType::WEBSOCKET as jint,
        "quic" => JavaRSocketTransportType::QUIC as jint,
        "iroh-p2p" => JavaRSocketTransportType::IROH_P2P as jint,
        _ => -1,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Transport_isTransportSupported(
    _env: JNIEnv,
    _class: JClass,
    transport_type: jint,
) -> jint {
    match transport_type {
        0 => 1, // TCP
        1 => 1, // WebSocket
        2 => 0, // QUIC - not yet supported in Java FFI
        3 => 0, // Iroh P2P - not yet supported in Java FFI
        _ => 0,
    }
}
