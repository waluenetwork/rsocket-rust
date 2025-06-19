use futures::{SinkExt, StreamExt};
use quinn::{RecvStream, SendStream};
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use tokio_util::codec::Framed;
use std::pin::Pin;
use std::task::{Context, Poll};

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
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.recv_stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for QuinnBiStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match Pin::new(&mut self.send_stream).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => Poll::Ready(Ok(n)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            ))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match Pin::new(&mut self.send_stream).poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            ))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match Pin::new(&mut self.send_stream).poll_shutdown(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            ))),
            Poll::Pending => Poll::Pending,
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
