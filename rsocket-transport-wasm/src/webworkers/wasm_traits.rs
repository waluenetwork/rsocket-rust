use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_util::{Sink, Stream};
use wasm_bindgen::prelude::*;

pub trait WasmTransport {
    type Conn: WasmConnection;
    
    fn connect(self) -> Pin<Box<dyn Future<Output = Result<Self::Conn, JsValue>>>>;
}

pub trait WasmConnection {
    fn split(self) -> (Box<dyn WasmFrameSink>, Box<dyn WasmFrameStream>);
}

pub trait WasmFrameSink {
    fn send(&mut self, frame: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<(), JsValue>>>>;
}

pub trait WasmFrameStream {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<Result<Vec<u8>, JsValue>>>>>;
}

#[derive(Debug)]
pub struct WasmFrame {
    pub data: Vec<u8>,
}

impl WasmFrame {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

pub struct WasmFrameSinkImpl {
    sender: web_sys::MessagePort,
}

impl WasmFrameSinkImpl {
    pub fn new(sender: web_sys::MessagePort) -> Self {
        Self { sender }
    }
}

impl WasmFrameSink for WasmFrameSinkImpl {
    fn send(&mut self, frame: Vec<u8>) -> Pin<Box<dyn Future<Output = Result<(), JsValue>>>> {
        let sender = self.sender.clone();
        Box::pin(async move {
            let array = js_sys::Uint8Array::new_with_length(frame.len() as u32);
            array.copy_from(&frame);
            sender.post_message(&array.into())?;
            Ok(())
        })
    }
}

pub struct WasmFrameStreamImpl {
    receiver: web_sys::MessagePort,
    pending_messages: std::collections::VecDeque<Vec<u8>>,
}

impl WasmFrameStreamImpl {
    pub fn new(receiver: web_sys::MessagePort) -> Self {
        Self {
            receiver,
            pending_messages: std::collections::VecDeque::new(),
        }
    }
}

impl WasmFrameStream for WasmFrameStreamImpl {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<Result<Vec<u8>, JsValue>>>>> {
        if let Some(message) = self.pending_messages.pop_front() {
            return Box::pin(async move { Some(Ok(message)) });
        }
        
        let receiver = self.receiver.clone();
        Box::pin(async move {
            None
        })
    }
}
