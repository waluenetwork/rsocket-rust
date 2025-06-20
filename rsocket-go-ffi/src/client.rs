use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Arc;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use tokio::runtime::Runtime;
use super::{GoRSocketPayload, GoCallback, create_error};

#[repr(C)]
pub struct GoRSocketClient {
    runtime: Box<Runtime>,
    client: Option<Arc<dyn RSocket>>,
}

#[no_mangle]
pub extern "C" fn rsocket_go_client_create() -> *mut GoRSocketClient {
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };
    
    Box::into_raw(Box::new(GoRSocketClient {
        runtime: Box::new(runtime),
        client: None,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_go_client_connect_tcp(
    client: *mut GoRSocketClient,
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
                client_ref.client = Some(Arc::new(rsocket));
                0
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_client_connect_websocket(
    client: *mut GoRSocketClient,
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
                client_ref.client = Some(Arc::new(rsocket));
                0
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_client_is_connected(client: *const GoRSocketClient) -> c_int {
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
pub extern "C" fn rsocket_go_client_request_response(
    client: *mut GoRSocketClient,
    payload: *mut GoRSocketPayload,
    callback: GoCallback,
    user_data: *mut c_void,
) -> c_int {
    if client.is_null() || payload.is_null() {
        return -1;
    }
    
    unsafe {
        let client_ref = &mut *client;
        if let Some(ref rsocket) = client_ref.client {
            let payload_inner = (*payload).inner().clone();
            let rsocket_clone = rsocket.clone();
            
            let user_data_usize = user_data as usize;
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    let user_data = user_data_usize as *mut c_void;
                    match rsocket_clone.request_response(payload_inner).await {
                        Ok(response) => {
                            if let Some(payload) = response {
                                let response_payload = Box::into_raw(Box::new(GoRSocketPayload {
                                    inner: Box::new(payload),
                                }));
                                callback(response_payload as *mut c_void, ptr::null_mut(), user_data);
                            } else {
                                callback(ptr::null_mut(), ptr::null_mut(), user_data);
                            }
                        }
                        Err(e) => {
                            let error = create_error(-1, &format!("{}", e));
                            callback(ptr::null_mut(), error, user_data);
                        }
                    }
                });
            });
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_client_fire_and_forget(
    client: *mut GoRSocketClient,
    payload: *mut GoRSocketPayload,
) -> c_int {
    if client.is_null() || payload.is_null() {
        return -1;
    }
    
    unsafe {
        let client_ref = &mut *client;
        if let Some(ref rsocket) = client_ref.client {
            let payload_inner = (*payload).inner().clone();
            let rsocket_clone = rsocket.clone();
            
            client_ref.runtime.spawn(async move {
                let _ = rsocket_clone.fire_and_forget(payload_inner).await;
            });
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_client_free(client: *mut GoRSocketClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}
