use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use tokio::runtime::Runtime;
use super::{RSocketPayload, RSocketError};

#[repr(C)]
pub struct RSocketClient {
    runtime: Box<Runtime>,
    client: Option<Box<dyn RSocket>>,
}

pub type RSocketResponseCallback = extern "C" fn(*mut RSocketPayload, *mut RSocketError, *mut c_void);
pub type RSocketStreamCallback = extern "C" fn(*mut RSocketPayload, c_int, *mut RSocketError, *mut c_void);

#[no_mangle]
pub extern "C" fn rsocket_client_create() -> *mut RSocketClient {
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };
    
    Box::into_raw(Box::new(RSocketClient {
        runtime: Box::new(runtime),
        client: None,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_client_connect_tcp(
    client: *mut RSocketClient,
    addr: *const c_char,
) -> c_int {
    if client.is_null() || addr.is_null() {
        return -1;
    }
    
    let addr_str = unsafe {
        match CStr::from_ptr(addr).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };
    
    unsafe {
        let client_ref = &mut *client;
        match client_ref.runtime.block_on(async {
            RSocketFactory::connect()
                .transport(TcpClientTransport::from(addr_str))
                .start()
                .await
        }) {
            Ok(rsocket) => {
                client_ref.client = Some(Box::new(rsocket));
                0
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_client_connect_websocket(
    client: *mut RSocketClient,
    url: *const c_char,
) -> c_int {
    if client.is_null() || url.is_null() {
        return -1;
    }
    
    let url_str = unsafe {
        match CStr::from_ptr(url).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };
    
    unsafe {
        let client_ref = &mut *client;
        match client_ref.runtime.block_on(async {
            RSocketFactory::connect()
                .transport(WebsocketClientTransport::from(url_str))
                .start()
                .await
        }) {
            Ok(rsocket) => {
                client_ref.client = Some(Box::new(rsocket));
                0
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_client_request_response(
    client: *mut RSocketClient,
    payload: *mut RSocketPayload,
    callback: RSocketResponseCallback,
    user_data: *mut c_void,
) -> c_int {
    if client.is_null() || payload.is_null() {
        return -1;
    }
    
    unsafe {
        let client_ref = &mut *client;
        if let Some(ref rsocket) = client_ref.client {
            let payload_inner = (*payload).to_rsocket_payload();
            let rsocket_clone = rsocket;
            
            let result = client_ref.runtime.block_on(async move {
                rsocket_clone.request_response(payload_inner).await
            });
            
            match result {
                Ok(Some(response)) => {
                    let response_payload = RSocketPayload::from_rsocket_payload(response);
                    callback(response_payload, ptr::null_mut(), user_data);
                }
                Ok(None) => {
                    let error_msg = CString::new("No response received").unwrap();
                    let error = Box::into_raw(Box::new(RSocketError {
                        code: -2,
                        message: error_msg.into_raw(),
                    }));
                    callback(ptr::null_mut(), error, user_data);
                }
                Err(e) => {
                    let error_msg = CString::new(format!("{}", e)).unwrap();
                    let error = Box::into_raw(Box::new(RSocketError {
                        code: -1,
                        message: error_msg.into_raw(),
                    }));
                    callback(ptr::null_mut(), error, user_data);
                }
            }
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_client_fire_and_forget(
    client: *mut RSocketClient,
    payload: *mut RSocketPayload,
) -> c_int {
    if client.is_null() || payload.is_null() {
        return -1;
    }
    
    unsafe {
        let client_ref = &mut *client;
        if let Some(ref rsocket) = client_ref.client {
            let payload_inner = (*payload).to_rsocket_payload();
            let rsocket_clone = rsocket;
            
            let _ = client_ref.runtime.block_on(async move {
                rsocket_clone.fire_and_forget(payload_inner).await
            });
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_client_is_connected(client: *const RSocketClient) -> c_int {
    if client.is_null() {
        return 0;
    }
    
    unsafe {
        if (*client).client.is_some() {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_client_free(client: *mut RSocketClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}
