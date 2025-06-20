use jni::JNIEnv;
use jni::objects::{JClass, JString, JByteArray, JObject};
use jni::sys::{jlong, jint, jbyteArray, jobject};
use rsocket_rust::prelude::Payload;
use bytes::Bytes;
use std::ptr;

#[repr(C)]
pub struct JavaRSocketPayload {
    inner: Box<Payload>,
}

impl JavaRSocketPayload {
    pub fn new(payload: Payload) -> Self {
        Self {
            inner: Box::new(payload),
        }
    }
    
    pub fn inner(&self) -> &Payload {
        &self.inner
    }
    
    pub fn into_inner(self) -> Payload {
        *self.inner
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Payload_create(
    env: JNIEnv,
    _class: JClass,
    data: jbyteArray,
    metadata: jbyteArray,
) -> jlong {
    let data_bytes = if !data.is_null() {
        match env.convert_byte_array(data) {
            Ok(bytes) => Some(Bytes::from(bytes)),
            Err(_) => return 0,
        }
    } else {
        None
    };
    
    let metadata_bytes = if !metadata.is_null() {
        match env.convert_byte_array(metadata) {
            Ok(bytes) => Some(Bytes::from(bytes)),
            Err(_) => return 0,
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
    
    let java_payload = Box::new(JavaRSocketPayload::new(payload));
    Box::into_raw(java_payload) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Payload_createFromString(
    env: JNIEnv,
    _class: JClass,
    data: JString,
    metadata: JString,
) -> jlong {
    let data_str = if !data.is_null() {
        match env.get_string(data) {
            Ok(s) => s.into(),
            Err(_) => return 0,
        }
    } else {
        String::new()
    };
    
    let metadata_str = if !metadata.is_null() {
        match env.get_string(metadata) {
            Ok(s) => Some(s.into()),
            Err(_) => return 0,
        }
    } else {
        None
    };
    
    let data_bytes = Bytes::from(data_str.into_bytes());
    let metadata_bytes = metadata_str.map(|s: String| Bytes::from(s.into_bytes()));
    
    let payload = match metadata_bytes {
        Some(metadata) => Payload::builder().set_data(data_bytes).set_metadata(metadata).build(),
        None => Payload::builder().set_data(data_bytes).build(),
    };
    
    let java_payload = Box::new(JavaRSocketPayload::new(payload));
    Box::into_raw(java_payload) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Payload_getDataLength(
    _env: JNIEnv,
    _class: JClass,
    payload_ptr: jlong,
) -> jint {
    if payload_ptr == 0 {
        return 0;
    }
    
    let payload = unsafe { &*(payload_ptr as *const JavaRSocketPayload) };
    payload.inner().data().map_or(0, |data| data.len() as jint)
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Payload_getData(
    env: JNIEnv,
    _class: JClass,
    payload_ptr: jlong,
) -> jbyteArray {
    if payload_ptr == 0 {
        return ptr::null_mut();
    }
    
    let payload = unsafe { &*(payload_ptr as *const JavaRSocketPayload) };
    if let Some(data) = payload.inner().data() {
        match env.byte_array_from_slice(data) {
            Ok(array) => array,
            Err(_) => ptr::null_mut(),
        }
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_Payload_free(
    _env: JNIEnv,
    _class: JClass,
    payload_ptr: jlong,
) {
    if payload_ptr != 0 {
        unsafe {
            let _ = Box::from_raw(payload_ptr as *mut JavaRSocketPayload);
        }
    }
}
