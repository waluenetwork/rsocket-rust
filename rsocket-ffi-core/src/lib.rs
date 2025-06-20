
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Arc;
use rsocket_rust::prelude::*;
use rsocket_rust::error::RSocketError;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;

#[repr(C)]
pub struct RSocketClient {
    inner: Arc<dyn RSocket>,
}

#[repr(C)]
pub struct RSocketPayload {
    data: *mut c_char,
    data_len: usize,
    metadata: *mut c_char,
    metadata_len: usize,
}

#[repr(C)]
pub struct RSocketTransportConfig {
    transport_type: c_int, // 0=TCP, 1=WebSocket, 2=QUIC, 3=Iroh, 4=WebTransport, 5=iroh-roq
    address: *const c_char,
    enable_advanced_features: c_int,
}

pub type RSocketCallback = extern "C" fn(*const RSocketPayload, *mut c_void);

#[no_mangle]
pub extern "C" fn rsocket_create_client(config: *const RSocketTransportConfig) -> *mut RSocketClient {
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
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = rt.block_on(async {
        match config.transport_type {
            0 => {
                let transport = TcpClientTransport::from(address);
                RSocketFactory::connect()
                    .transport(transport)
                    .start()
                    .await
                    .map_err(|e| RSocketError::Other(e.into()))
            }
            1 => {
                let transport = WebsocketClientTransport::from(address.as_str());
                RSocketFactory::connect()
                    .transport(transport)
                    .start()
                    .await
                    .map_err(|e| RSocketError::Other(e.into()))
            }
            _ => Err(RSocketError::Other(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unsupported transport type: {}", config.transport_type)
            )).into())),
        }
    });
    
    match client {
        Ok(client) => Box::into_raw(Box::new(RSocketClient { inner: Arc::new(client) })),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rsocket_request_response_async(
    client: *mut RSocketClient,
    payload: *const RSocketPayload,
    callback: RSocketCallback,
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
        CStr::from_ptr((*payload).data).to_string_lossy().into_owned()
    };
    
    let client_clone = client.inner.clone();
    
    let callback_ptr = callback as usize;
    let user_data_ptr = user_data as usize;
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let callback: RSocketCallback = unsafe { std::mem::transmute(callback_ptr) };
            let user_data = user_data_ptr as *mut c_void;
            
            let payload = Payload::builder()
                .set_data_utf8(&payload_data)
                .build();
            
            match client_clone.request_response(payload).await {
                Ok(Some(response)) => {
                    let data = response.data_utf8().unwrap_or_default();
                    let c_data = CString::new(data).unwrap();
                    let c_payload = RSocketPayload {
                        data: c_data.into_raw(),
                        data_len: data.len(),
                        metadata: ptr::null_mut(),
                        metadata_len: 0,
                    };
                    callback(&c_payload, user_data);
                }
                Ok(None) => {
                    callback(ptr::null(), user_data);
                }
                Err(_) => {
                    callback(ptr::null(), user_data);
                }
            }
        })
    });
    
    0
}

#[no_mangle]
pub extern "C" fn rsocket_free_client(client: *mut RSocketClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_free_payload(payload: *mut RSocketPayload) {
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
