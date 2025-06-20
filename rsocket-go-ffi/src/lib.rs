use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;

use std::sync::Arc;
use tokio::runtime::Runtime;

#[repr(C)]
pub struct CRSocketClient {
    inner: Arc<dyn RSocket>,
    runtime: Arc<Runtime>,
}

#[no_mangle]
pub extern "C" fn rsocket_go_create_tcp_client(address: *const c_char) -> *mut CRSocketClient {
    if address.is_null() {
        return ptr::null_mut();
    }
    
    let address = unsafe { CStr::from_ptr(address).to_string_lossy().into_owned() };
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = rt.block_on(async {
        let transport = TcpClientTransport::from(address);
        RSocketFactory::connect()
            .transport(transport)
            .start()
            .await
    });
    
    match client {
        Ok(client) => Box::into_raw(Box::new(CRSocketClient { 
            inner: Arc::new(client),
            runtime: Arc::new(rt),
        })),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_create_websocket_client(address: *const c_char) -> *mut CRSocketClient {
    if address.is_null() {
        return ptr::null_mut();
    }
    
    let address = unsafe { CStr::from_ptr(address).to_string_lossy().into_owned() };
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = rt.block_on(async {
        let transport = WebsocketClientTransport::from(address.as_str());
        RSocketFactory::connect()
            .transport(transport)
            .start()
            .await
    });
    
    match client {
        Ok(client) => Box::into_raw(Box::new(CRSocketClient { 
            inner: Arc::new(client),
            runtime: Arc::new(rt),
        })),
        Err(_) => ptr::null_mut(),
    }
}



#[no_mangle]
pub extern "C" fn rsocket_go_request_response_sync(
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
        std::slice::from_raw_parts(data as *const u8, data_len)
    };
    let payload_str = String::from_utf8_lossy(payload_data);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let payload = Payload::builder()
            .set_data_utf8(&payload_str)
            .build();
        
        client.inner.request_response(payload).await
    });
    
    match result {
        Ok(Some(response)) => {
            let data = response.data_utf8().unwrap_or_default();
            let c_data = CString::new(data).unwrap();
            unsafe {
                *response_len = data.len();
                *response_data = c_data.into_raw();
            }
            0
        }
        Ok(None) | Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_fire_and_forget(
    client: *mut CRSocketClient,
    data: *const c_char,
    data_len: usize,
) -> c_int {
    if client.is_null() || data.is_null() {
        return -1;
    }
    
    let client = unsafe { &*client };
    let payload_data = unsafe {
        std::slice::from_raw_parts(data as *const u8, data_len)
    };
    let payload_str = String::from_utf8_lossy(payload_data);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let payload = Payload::builder()
            .set_data_utf8(&payload_str)
            .build();
        
        client.inner.fire_and_forget(payload).await
    });
    
    match result {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
