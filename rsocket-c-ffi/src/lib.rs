
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;

#[repr(C)]
pub struct CRSocketClient {
    inner: Arc<dyn RSocket>,
    runtime: Arc<Runtime>,
}

#[repr(C)]
pub struct CRSocketPayload {
    data: *mut c_char,
    data_len: usize,
    metadata: *mut c_char,
    metadata_len: usize,
}

#[repr(C)]
pub struct CRSocketTransportConfig {
    transport_type: c_int, // 0=TCP, 1=WebSocket, 2=WASM
    address: *const c_char,
    enable_advanced_features: c_int,
}

pub type CRSocketCallback = extern "C" fn(*const CRSocketPayload, *mut c_void);

#[no_mangle]
pub extern "C" fn rsocket_c_create_client(config: *const CRSocketTransportConfig) -> *mut CRSocketClient {
    if config.is_null() {
        return ptr::null_mut();
    }
    
    let config = unsafe { &*config };
    let address = unsafe {
        if config.address.is_null() {
            return ptr::null_mut();
        }
        CStr::from_ptr(config.address).to_string_lossy().into_owned()
    };
    
    let runtime = match Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(_) => return ptr::null_mut(),
    };
    
    let client = runtime.block_on(async {
        match config.transport_type {
            0 => {
                let transport = TcpClientTransport::from(address);
                RSocketFactory::connect()
                    .transport(transport)
                    .start()
                    .await
                    .ok()
            }
            1 => {
                let transport = WebsocketClientTransport::from(address.as_str());
                RSocketFactory::connect()
                    .transport(transport)
                    .start()
                    .await
                    .ok()
            }
            _ => None,
        }
    });
    
    match client {
        Some(rsocket) => {
            Box::into_raw(Box::new(CRSocketClient {
                inner: Arc::new(rsocket),
                runtime,
            }))
        }
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rsocket_c_request_response_async(
    client: *mut CRSocketClient,
    payload: *const CRSocketPayload,
    callback: CRSocketCallback,
    user_data: *mut c_void,
) -> c_int {
    if client.is_null() || payload.is_null() {
        return -1;
    }
    
    let client = unsafe { &*client };
    let payload_data = unsafe {
        if (*payload).data.is_null() {
            return -1;
        }
        let slice = std::slice::from_raw_parts((*payload).data as *const u8, (*payload).data_len);
        String::from_utf8_lossy(slice).into_owned()
    };
    
    let inner = client.inner.clone();
    let runtime = client.runtime.clone();
    
    let result = runtime.block_on(async move {
        let payload = Payload::builder()
            .set_data_utf8(&payload_data)
            .build();
        
        inner.request_response(payload).await
    });
    
    match result {
        Ok(Some(response)) => {
            let data = response.data_utf8().unwrap_or_default();
            let c_data = match CString::new(data) {
                Ok(s) => s.into_raw(),
                Err(_) => ptr::null_mut(),
            };
            let c_payload = CRSocketPayload {
                data: c_data,
                data_len: data.len(),
                metadata: ptr::null_mut(),
                metadata_len: 0,
            };
            callback(&c_payload, user_data);
        }
        Ok(None) | Err(_) => {
            callback(ptr::null(), user_data);
        }
    }
    
    0
}

#[no_mangle]
pub extern "C" fn rsocket_c_request_response_sync(
    client: *mut CRSocketClient,
    data: *const c_char,
    data_len: usize,
    response_data: *mut *mut c_char,
    response_len: *mut usize,
) -> c_int {
    if client.is_null() || data.is_null() || response_data.is_null() || response_len.is_null() {
        return -1;
    }
    
    let client = unsafe { &*client };
    let payload_data = unsafe {
        let slice = std::slice::from_raw_parts(data as *const u8, data_len);
        String::from_utf8_lossy(slice).into_owned()
    };
    
    let result = client.runtime.block_on(async {
        let payload = Payload::builder()
            .set_data_utf8(&payload_data)
            .build();
        
        client.inner.request_response(payload).await
    });
    
    match result {
        Ok(Some(response)) => {
            let data = response.data_utf8().unwrap_or_default();
            match CString::new(data) {
                Ok(c_data) => {
                    unsafe {
                        *response_len = data.len();
                        *response_data = c_data.into_raw();
                    }
                    0
                }
                Err(_) => -1,
            }
        }
        Ok(None) | Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rsocket_c_free_client(client: *mut CRSocketClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_c_free_payload(payload: *mut CRSocketPayload) {
    if !payload.is_null() {
        unsafe {
            let payload = &*payload;
            if !payload.data.is_null() {
                let _ = CString::from_raw(payload.data);
            }
            if !payload.metadata.is_null() {
                let _ = CString::from_raw(payload.metadata);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_c_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
