use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
#[cfg(target_arch = "wasm32")]
use rsocket_rust::error::RSocketError;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{ReadableStream, WritableStream};
#[cfg(target_arch = "wasm32")]
use bytes::{Buf, BufMut};

#[derive(Debug)]
pub enum WebTransportConnection {
    #[cfg(not(target_arch = "wasm32"))]
    Native(crate::connection::QuinnConnection),
    #[cfg(target_arch = "wasm32")]
    Browser {
        readable: ReadableStream,
        writable: WritableStream,
    },
}

impl WebTransportConnection {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_quinn(quinn_conn: crate::connection::QuinnConnection) -> Self {
        Self::Native(quinn_conn)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_streams(readable: ReadableStream, writable: WritableStream) -> Self {
        Self::Browser { readable, writable }
    }
}

impl Connection for WebTransportConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            WebTransportConnection::Native(quinn_conn) => quinn_conn.split(),
            #[cfg(target_arch = "wasm32")]
            WebTransportConnection::Browser { readable, writable } => {
                let sink = WebTransportSink::new(writable);
                let stream = WebTransportStream::new(readable);
                
                (
                    Box::new(sink),
                    Box::new(stream),
                )
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct WebTransportSink {
    writable: WritableStream,
}

#[cfg(target_arch = "wasm32")]
impl WebTransportSink {
    fn new(writable: WritableStream) -> Self {
        Self { writable }
    }
}

#[cfg(target_arch = "wasm32")]
impl futures::Sink<rsocket_rust::frame::Frame> for WebTransportSink {
    type Error = RSocketError;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: rsocket_rust::frame::Frame) -> Result<(), Self::Error> {
        use bytes::Buf;
        
        log::debug!("WebTransport sending frame: {:?}", item);
        
        let mut buf = bytes::BytesMut::new();
        item.write_to(&mut buf);
        let frame_bytes = buf.freeze();
        
        let length_bytes = (frame_bytes.len() as u32).to_be_bytes();
        let mut full_frame = bytes::BytesMut::with_capacity(4 + frame_bytes.len());
        full_frame.extend_from_slice(&length_bytes);
        full_frame.extend_from_slice(&frame_bytes);
        
        let writer = self.writable.get_writer()
            .map_err(|e| RSocketError::Other(format!("Failed to get writer: {:?}", e).into()))?;
        
        let uint8_array = js_sys::Uint8Array::new_with_length(full_frame.len() as u32);
        uint8_array.copy_from(&full_frame);
        
        let write_promise = writer.write_with_chunk(&uint8_array.into());
        
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = wasm_bindgen_futures::JsFuture::from(write_promise).await {
                log::error!("WebTransport write failed: {:?}", e);
            }
        });
        
        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let writer = match self.writable.get_writer() {
            Ok(writer) => writer,
            Err(_) => return std::task::Poll::Ready(Ok(())),
        };
        
        let close_promise = writer.close();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(close_promise).await;
        });
        
        std::task::Poll::Ready(Ok(()))
    }
}

#[cfg(target_arch = "wasm32")]
struct WebTransportStream {
    readable: ReadableStream,
    reader: Option<web_sys::ReadableStreamDefaultReader>,
    buffer: bytes::BytesMut,
}

#[cfg(target_arch = "wasm32")]
impl WebTransportStream {
    fn new(readable: ReadableStream) -> Self {
        Self { 
            readable,
            reader: None,
            buffer: bytes::BytesMut::new(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl futures::Stream for WebTransportStream {
    type Item = Result<rsocket_rust::frame::Frame, RSocketError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use bytes::Buf;
        
        if self.reader.is_none() {
            let reader = match self.readable.get_reader() {
                Ok(reader) => reader,
                Err(e) => return std::task::Poll::Ready(Some(Err(
                    RSocketError::Other(format!("Failed to get reader: {:?}", e).into())
                ))),
            };
            self.reader = Some(reader);
        }
        
        if self.buffer.len() >= 4 {
            let frame_length = u32::from_be_bytes([
                self.buffer[0], self.buffer[1], self.buffer[2], self.buffer[3]
            ]) as usize;
            
            if self.buffer.len() >= 4 + frame_length {
                self.buffer.advance(4);
                let frame_data = self.buffer.split_to(frame_length);
                
                match rsocket_rust::frame::Frame::read_from(&mut frame_data.as_ref()) {
                    Ok(frame) => return std::task::Poll::Ready(Some(Ok(frame))),
                    Err(e) => return std::task::Poll::Ready(Some(Err(e))),
                }
            }
        }
        
        if let Some(reader) = &self.reader {
            let read_promise = reader.read();
            let future = wasm_bindgen_futures::JsFuture::from(read_promise);
            
            match futures::ready!(Box::pin(future).as_mut().poll(cx)) {
                Ok(result) => {
                    let done = js_sys::Reflect::get(&result, &"done".into())
                        .unwrap_or(wasm_bindgen::JsValue::from(false))
                        .as_bool()
                        .unwrap_or(false);
                    
                    if done {
                        return std::task::Poll::Ready(None);
                    }
                    
                    let value = js_sys::Reflect::get(&result, &"value".into())
                        .unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
                    
                    if let Ok(uint8_array) = value.dyn_into::<js_sys::Uint8Array>() {
                        let mut data = vec![0u8; uint8_array.length() as usize];
                        uint8_array.copy_to(&mut data);
                        self.buffer.extend_from_slice(&data);
                        
                        cx.waker().wake_by_ref();
                        std::task::Poll::Pending
                    } else {
                        std::task::Poll::Pending
                    }
                }
                Err(e) => std::task::Poll::Ready(Some(Err(
                    RSocketError::Other(format!("Read failed: {:?}", e).into())
                ))),
            }
        } else {
            std::task::Poll::Pending
        }
    }
}
