use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpServerTransport;
use tokio::runtime::Runtime;
use super::RSocketPayload;

#[repr(C)]
pub struct RSocketServer {
    runtime: Box<Runtime>,
    server: Option<Box<dyn ServerTransport<Item = Box<dyn RSocket>>>>,
}

pub type RSocketRequestHandler = extern "C" fn(*mut RSocketPayload, *mut c_void) -> *mut RSocketPayload;

#[no_mangle]
pub extern "C" fn rsocket_server_create() -> *mut RSocketServer {
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };
    
    Box::into_raw(Box::new(RSocketServer {
        runtime: Box::new(runtime),
        server: None,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_server_bind_tcp(
    server: *mut RSocketServer,
    addr: *const c_char,
    _handler: RSocketRequestHandler,
    _user_data: *mut c_void,
) -> c_int {
    if server.is_null() || addr.is_null() {
        return -1;
    }
    
    let addr_str = unsafe {
        match CStr::from_ptr(addr).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };
    
    unsafe {
        let server_ref = &mut *server;
        match server_ref.runtime.block_on(async {
            let mut transport = TcpServerTransport::from(addr_str);
            transport.start().await.map_err(|e| format!("{}", e))?;
            Ok::<TcpServerTransport, String>(transport)
        }) {
            Ok(_transport) => {
                0
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_server_start(server: *mut RSocketServer) -> c_int {
    if server.is_null() {
        return -1;
    }
    
    unsafe {
        let server_ref = &mut *server;
        if server_ref.server.is_some() {
            0
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_server_stop(server: *mut RSocketServer) -> c_int {
    if server.is_null() {
        return -1;
    }
    
    unsafe {
        let server_ref = &mut *server;
        server_ref.server = None;
        0
    }
}

#[no_mangle]
pub extern "C" fn rsocket_server_free(server: *mut RSocketServer) {
    if !server.is_null() {
        unsafe {
            let _ = Box::from_raw(server);
        }
    }
}
