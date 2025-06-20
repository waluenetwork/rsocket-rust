use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject, JValue};
use jni::sys::{jlong, jint, jstring, jobject, jboolean};
use std::ptr;

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
pub extern "system" fn Java_com_rsocket_rust_RSocket_init(env: JNIEnv, _class: JClass) -> jint {
    env_logger::try_init().unwrap_or(());
    0
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocket_getVersion(env: JNIEnv, _class: JClass) -> jstring {
    let version = env!("CARGO_PKG_VERSION");
    let output = env.new_string(version).expect("Couldn't create java string!");
    output.into_inner()
}

#[repr(C)]
pub struct JavaRSocketError {
    pub code: jint,
    pub message: String,
}

impl JavaRSocketError {
    pub fn new(code: jint, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
    
    pub fn to_java_exception(&self, env: &JNIEnv) -> Result<(), jni::errors::Error> {
        let exception_class = env.find_class("com/rsocket/rust/RSocketException")?;
        let message = env.new_string(&self.message)?;
        env.throw_new(exception_class, message.to_str()?)?;
        Ok(())
    }
}

pub struct JavaCallback {
    pub env: *mut jni::sys::JNIEnv,
    pub callback_object: jni::objects::GlobalRef,
}

unsafe impl Send for JavaCallback {}
unsafe impl Sync for JavaCallback {}

impl JavaCallback {
    pub fn new(env: &JNIEnv, callback_object: JObject) -> Result<Self, jni::errors::Error> {
        let global_ref = env.new_global_ref(callback_object)?;
        Ok(Self {
            env: env.get_native_interface() as *mut jni::sys::JNIEnv,
            callback_object: global_ref,
        })
    }
    
    pub fn call_response(&self, response: Option<&JavaRSocketPayload>, error: Option<&JavaRSocketError>) {
    }
}

pub fn jstring_to_string(env: &JNIEnv, jstr: JString) -> Result<String, jni::errors::Error> {
    Ok(env.get_string(jstr)?.into())
}

pub fn string_to_jstring(env: &JNIEnv, s: &str) -> Result<jstring, jni::errors::Error> {
    let jstr = env.new_string(s)?;
    Ok(jstr.into_inner())
}
