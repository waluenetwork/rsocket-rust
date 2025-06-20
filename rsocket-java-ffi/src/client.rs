use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jlong, jint, jobject};
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use tokio::runtime::Runtime;
use super::{JavaRSocketPayload, JavaRSocketError, JavaCallback};
use std::ptr;

#[repr(C)]
pub struct JavaRSocketClient {
    runtime: Box<Runtime>,
    client: Option<Box<dyn RSocket>>,
}

impl JavaRSocketClient {
    pub fn new() -> Result<Self, JavaRSocketError> {
        let runtime = Runtime::new()
            .map_err(|e| JavaRSocketError::new(-1, &format!("Failed to create runtime: {}", e)))?;
        
        Ok(Self {
            runtime: Box::new(runtime),
            client: None,
        })
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketClient_create(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    match JavaRSocketClient::new() {
        Ok(client) => Box::into_raw(Box::new(client)) as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketClient_connectTcp(
    env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
    address: JString,
) -> jint {
    if client_ptr == 0 {
        return -1;
    }
    
    let addr_str = match env.get_string(address) {
        Ok(s) => s.into(),
        Err(_) => return -1,
    };
    
    let client = unsafe { &mut *(client_ptr as *mut JavaRSocketClient) };
    
    match client.runtime.block_on(async {
        RSocketFactory::connect()
            .transport(TcpClientTransport::from(addr_str))
            .start()
            .await
    }) {
        Ok(rsocket) => {
            client.client = Some(Box::new(rsocket));
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketClient_connectWebSocket(
    env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
    url: JString,
) -> jint {
    if client_ptr == 0 {
        return -1;
    }
    
    let url_str = match env.get_string(url) {
        Ok(s) => s.into(),
        Err(_) => return -1,
    };
    
    let client = unsafe { &mut *(client_ptr as *mut JavaRSocketClient) };
    
    match client.runtime.block_on(async {
        RSocketFactory::connect()
            .transport(WebsocketClientTransport::from(url_str))
            .start()
            .await
    }) {
        Ok(rsocket) => {
            client.client = Some(Box::new(rsocket));
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketClient_requestResponse(
    env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
    payload_ptr: jlong,
    callback: JObject,
) -> jint {
    if client_ptr == 0 || payload_ptr == 0 {
        return -1;
    }
    
    let client = unsafe { &mut *(client_ptr as *mut JavaRSocketClient) };
    let payload = unsafe { &*(payload_ptr as *const JavaRSocketPayload) };
    
    if let Some(ref rsocket) = client.client {
        let payload_inner = payload.inner().clone();
        let java_callback = match JavaCallback::new(&env, callback) {
            Ok(cb) => cb,
            Err(_) => return -1,
        };
        
        client.runtime.spawn(async move {
            match rsocket.request_response(payload_inner).await {
                Ok(response) => {
                    let response_payload = JavaRSocketPayload::new(response);
                    java_callback.call_response(Some(&response_payload), None);
                }
                Err(e) => {
                    let error = JavaRSocketError::new(-1, &format!("{}", e));
                    java_callback.call_response(None, Some(&error));
                }
            }
        });
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketClient_fireAndForget(
    _env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
    payload_ptr: jlong,
) -> jint {
    if client_ptr == 0 || payload_ptr == 0 {
        return -1;
    }
    
    let client = unsafe { &mut *(client_ptr as *mut JavaRSocketClient) };
    let payload = unsafe { &*(payload_ptr as *const JavaRSocketPayload) };
    
    if let Some(ref rsocket) = client.client {
        let payload_inner = payload.inner().clone();
        
        client.runtime.spawn(async move {
            let _ = rsocket.fire_and_forget(payload_inner).await;
        });
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketClient_free(
    _env: JNIEnv,
    _class: JClass,
    client_ptr: jlong,
) {
    if client_ptr != 0 {
        unsafe {
            let _ = Box::from_raw(client_ptr as *mut JavaRSocketClient);
        }
    }
}
