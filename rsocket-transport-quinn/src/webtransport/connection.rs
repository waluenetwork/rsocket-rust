use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
#[cfg(target_arch = "wasm32")]
use rsocket_rust::error::RSocketError;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{ReadableStream, WritableStream};

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
        log::debug!("WebTransport sending frame: {:?}", item);
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
        std::task::Poll::Ready(Ok(()))
    }
}

#[cfg(target_arch = "wasm32")]
struct WebTransportStream {
    readable: ReadableStream,
}

#[cfg(target_arch = "wasm32")]
impl WebTransportStream {
    fn new(readable: ReadableStream) -> Self {
        Self { readable }
    }
}

#[cfg(target_arch = "wasm32")]
impl futures::Stream for WebTransportStream {
    type Item = Result<rsocket_rust::frame::Frame, RSocketError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Pending
    }
}
