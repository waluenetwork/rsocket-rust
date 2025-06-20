use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};

mod client;
mod server;
mod transport;
mod payload;
mod performance;

pub use client::*;
pub use server::*;
pub use transport::*;
pub use payload::*;
pub use performance::*;

#[no_mangle]
pub extern "C" fn rsocket_go_init() -> c_int {
    env_logger::try_init().unwrap_or(());
    0
}

#[no_mangle]
pub extern "C" fn rsocket_go_get_version() -> *const c_char {
    let version = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
    version.into_raw()
}

#[no_mangle]
pub extern "C" fn rsocket_go_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[repr(C)]
pub struct GoRSocketError {
    pub code: c_int,
    pub message: *const c_char,
}

#[no_mangle]
pub extern "C" fn rsocket_go_error_free(error: *mut GoRSocketError) {
    if !error.is_null() {
        unsafe {
            rsocket_go_free_string((*error).message as *mut c_char);
            let _ = Box::from_raw(error);
        }
    }
}

pub type GoCallback = extern "C" fn(*mut c_void, *mut GoRSocketError, *mut c_void);
pub type GoStreamCallback = extern "C" fn(*mut c_void, c_int, *mut GoRSocketError, *mut c_void);

fn create_error(code: c_int, message: &str) -> *mut GoRSocketError {
    let error_msg = CString::new(message).unwrap();
    Box::into_raw(Box::new(GoRSocketError {
        code,
        message: error_msg.into_raw(),
    }))
}
