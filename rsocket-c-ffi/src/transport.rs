use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub enum RSocketTransportType {
    TCP = 0,
    WEBSOCKET = 1,
    QUIC = 2,
    IrohP2P = 3,
}

#[no_mangle]
pub extern "C" fn rsocket_get_supported_transports() -> *const c_char {
    let transports = CString::new("tcp,websocket").unwrap();
    transports.into_raw()
}

#[no_mangle]
pub extern "C" fn rsocket_transport_type_to_string(transport_type: RSocketTransportType) -> *const c_char {
    let type_str = match transport_type {
        RSocketTransportType::TCP => "tcp",
        RSocketTransportType::WEBSOCKET => "websocket",
        RSocketTransportType::QUIC => "quic",
        RSocketTransportType::IrohP2P => "iroh-p2p",
    };
    
    let c_str = CString::new(type_str).unwrap();
    c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn rsocket_parse_transport_type(type_str: *const c_char) -> RSocketTransportType {
    if type_str.is_null() {
        return RSocketTransportType::TCP;
    }
    
    let transport_str = unsafe {
        match CStr::from_ptr(type_str).to_str() {
            Ok(s) => s,
            Err(_) => return RSocketTransportType::TCP,
        }
    };
    
    match transport_str.to_lowercase().as_str() {
        "tcp" => RSocketTransportType::TCP,
        "websocket" | "ws" => RSocketTransportType::WEBSOCKET,
        "quic" => RSocketTransportType::QUIC,
        "iroh-p2p" | "iroh" => RSocketTransportType::IrohP2P,
        _ => RSocketTransportType::TCP,
    }
}

#[no_mangle]
pub extern "C" fn rsocket_is_transport_supported(transport_type: RSocketTransportType) -> c_int {
    match transport_type {
        RSocketTransportType::TCP | RSocketTransportType::WEBSOCKET => 1,
        RSocketTransportType::QUIC | RSocketTransportType::IrohP2P => 0,
    }
}
