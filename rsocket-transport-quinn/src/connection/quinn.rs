use futures::{SinkExt, StreamExt};
use quinn::{RecvStream, SendStream};
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use tokio_util::codec::Framed;

use super::codec::LengthBasedFrameCodec;

#[derive(Debug)]
pub struct QuinnConnection {
    send_stream: SendStream,
    recv_stream: RecvStream,
}

impl QuinnConnection {
    pub fn new(send_stream: SendStream, recv_stream: RecvStream) -> Self {
        Self { send_stream, recv_stream }
    }
}

pub struct QuinnBiStream {
    send_stream: SendStream,
    recv_stream: RecvStream,
}

impl QuinnBiStream {
    pub fn new(send_stream: SendStream, recv_stream: RecvStream) -> Self {
        Self { send_stream, recv_stream }
    }
}

impl tokio::io::AsyncRead for QuinnBiStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.recv_stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for QuinnBiStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match std::pin::Pin::new(&mut self.send_stream).poll_write(cx, buf) {
            std::task::Poll::Ready(Ok(n)) => std::task::Poll::Ready(Ok(n)),
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            ))),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match std::pin::Pin::new(&mut self.send_stream).poll_flush(cx) {
            std::task::Poll::Ready(Ok(())) => std::task::Poll::Ready(Ok(())),
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            ))),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match std::pin::Pin::new(&mut self.send_stream).poll_shutdown(cx) {
            std::task::Poll::Ready(Ok(())) => std::task::Poll::Ready(Ok(())),
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            ))),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl Connection for QuinnConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let bi_stream = QuinnBiStream::new(self.send_stream, self.recv_stream);
        let (sink, stream) = Framed::new(bi_stream, LengthBasedFrameCodec).split();
        
        (
            Box::new(sink.sink_map_err(|e| RSocketError::Other(e.into()))),
            Box::new(stream.map(|next| next.map_err(|e| RSocketError::Other(e.into())))),
        )
    }
}
