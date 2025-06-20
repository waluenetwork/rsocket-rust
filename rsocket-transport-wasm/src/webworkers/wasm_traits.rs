//! 

use wasm_bindgen::prelude::*;
use std::pin::Pin;
use std::future::Future;

pub trait WasmTransport {
    type Conn: WasmConnection;
    
    fn connect(self) -> Pin<Box<dyn Future<Output = Result<Self::Conn, JsValue>>>>;
}

pub trait WasmConnection {
    fn split(self) -> (Box<dyn WasmFrameSink>, Box<dyn WasmFrameStream>);
}

pub trait WasmFrameSink {
    fn send(&mut self, frame: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<(), JsValue>> + '_>>;
}

pub trait WasmFrameStream {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<Result<Vec<u8>, JsValue>>> + '_>>;
}

pub trait WasmFrameProcessor {
    fn process_frame(&mut self, frame: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, JsValue>>>>;
    fn process_batch(&mut self, frames: Vec<Vec<u8>>) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<u8>>, JsValue>>>>;
}

#[derive(Debug)]
pub struct WasmError {
    pub message: String,
    pub code: Option<u32>,
}

impl From<JsValue> for WasmError {
    fn from(js_val: JsValue) -> Self {
        let message = js_val.as_string().unwrap_or_else(|| "Unknown WASM error".to_string());
        WasmError {
            message,
            code: None,
        }
    }
}

impl Into<JsValue> for WasmError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.message)
    }
}
