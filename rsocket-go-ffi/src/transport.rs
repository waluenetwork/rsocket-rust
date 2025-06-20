use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[no_mangle]
pub extern "C" fn rsocket_go_get_supported_transports() -> *const c_char {
    let transports = CString::new("tcp,websocket").unwrap();
    transports.into_raw()
}

#[no_mangle]
pub extern "C" fn rsocket_go_validate_tcp_address(addr: *const c_char) -> c_int {
    if addr.is_null() {
        return 0;
    }
    
    let addr_str = unsafe {
        match CStr::from_ptr(addr).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };
    
    match addr_str.parse::<std::net::SocketAddr>() {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_validate_websocket_url(url: *const c_char) -> c_int {
    if url.is_null() {
        return 0;
    }
    
    let url_str = unsafe {
        match CStr::from_ptr(url).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };
    
    if url_str.starts_with("ws://") || url_str.starts_with("wss://") {
        1
    } else {
        0
    }
}
