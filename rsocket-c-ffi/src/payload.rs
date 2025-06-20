use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use libc::size_t;
use rsocket_rust::prelude::Payload;
use bytes::Bytes;

#[repr(C)]
pub struct RSocketPayload {
    inner: *mut std::ffi::c_void,
}

#[no_mangle]
pub extern "C" fn rsocket_payload_create(
    data: *const u8,
    data_len: size_t,
    metadata: *const u8,
    metadata_len: size_t,
) -> *mut RSocketPayload {
    let data_bytes = if !data.is_null() && data_len > 0 {
        unsafe {
            Some(Bytes::copy_from_slice(std::slice::from_raw_parts(data, data_len)))
        }
    } else {
        None
    };
    
    let metadata_bytes = if !metadata.is_null() && metadata_len > 0 {
        unsafe {
            Some(Bytes::copy_from_slice(std::slice::from_raw_parts(metadata, metadata_len)))
        }
    } else {
        None
    };
    
    let payload = match (data_bytes, metadata_bytes) {
        (Some(data), Some(metadata)) => Payload::builder().set_data(data).set_metadata(metadata).build(),
        (Some(data), None) => Payload::builder().set_data(data).build(),
        (None, Some(metadata)) => Payload::builder().set_metadata(metadata).build(),
        (None, None) => Payload::builder().build(),
    };
    
    Box::into_raw(Box::new(RSocketPayload {
        inner: Box::into_raw(Box::new(payload)) as *mut std::ffi::c_void,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_payload_create_from_string(
    data: *const c_char,
    metadata: *const c_char,
) -> *mut RSocketPayload {
    let data_str = if !data.is_null() {
        unsafe { CStr::from_ptr(data).to_string_lossy().into_owned() }
    } else {
        String::new()
    };
    
    let metadata_str = if !metadata.is_null() {
        Some(unsafe { CStr::from_ptr(metadata).to_string_lossy().into_owned() })
    } else {
        None
    };
    
    let data_bytes = Bytes::from(data_str.into_bytes());
    let metadata_bytes = metadata_str.map(|s| Bytes::from(s.into_bytes()));
    
    let payload = match metadata_bytes {
        Some(metadata) => Payload::builder().set_data(data_bytes).set_metadata(metadata).build(),
        None => Payload::builder().set_data(data_bytes).build(),
    };
    
    Box::into_raw(Box::new(RSocketPayload {
        inner: Box::into_raw(Box::new(payload)) as *mut std::ffi::c_void,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_payload_get_data_length(payload: *const RSocketPayload) -> size_t {
    if payload.is_null() {
        return 0;
    }
    unsafe {
        let payload_ref = &*((*payload).inner as *const Payload);
        payload_ref.data().map_or(0, |data| data.len())
    }
}

#[no_mangle]
pub extern "C" fn rsocket_payload_get_metadata_length(payload: *const RSocketPayload) -> size_t {
    if payload.is_null() {
        return 0;
    }
    unsafe {
        let payload_ref = &*((*payload).inner as *const Payload);
        payload_ref.metadata().map_or(0, |metadata| metadata.len())
    }
}

#[no_mangle]
pub extern "C" fn rsocket_payload_copy_data(
    payload: *const RSocketPayload,
    buffer: *mut u8,
    buffer_len: size_t,
) -> size_t {
    if payload.is_null() || buffer.is_null() {
        return 0;
    }
    
    unsafe {
        let payload_ref = &*((*payload).inner as *const Payload);
        if let Some(data) = payload_ref.data() {
            let copy_len = std::cmp::min(data.len(), buffer_len);
            std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, copy_len);
            copy_len
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_payload_free(payload: *mut RSocketPayload) {
    if !payload.is_null() {
        unsafe {
            let payload_box = Box::from_raw(payload);
            let _ = Box::from_raw(payload_box.inner as *mut Payload);
        }
    }
}

impl RSocketPayload {
    pub fn to_rsocket_payload(&self) -> Payload {
        unsafe {
            let payload_ref = &*(self.inner as *const Payload);
            payload_ref.clone()
        }
    }
    
    pub fn from_rsocket_payload(payload: Payload) -> *mut RSocketPayload {
        Box::into_raw(Box::new(RSocketPayload {
            inner: Box::into_raw(Box::new(payload)) as *mut std::ffi::c_void,
        }))
    }
}
