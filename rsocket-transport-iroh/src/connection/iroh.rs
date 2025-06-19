use futures::{SinkExt, StreamExt};
use quinn::{RecvStream, SendStream, Connection};
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection as RSocketConnection, FrameSink, FrameStream};
use tokio_util::codec::Framed;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::codec::LengthBasedFrameCodec;

#[derive(Debug)]
pub struct P2PConnection {
    send_stream: SendStream,
    recv_stream: RecvStream,
}

impl P2PConnection {
    pub fn new(send_stream: SendStream, recv_stream: RecvStream) -> Self {
        Self { send_stream, recv_stream }
    }
    
    pub async fn from_quinn_connection(connection: Connection) -> rsocket_rust::Result<Self> {
        let (send_stream, recv_stream) = connection.open_bi()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to open bidirectional stream: {}", e))?;
        Ok(Self::new(send_stream, recv_stream))
    }
}

pub struct P2PBiStream {
    send_stream: SendStream,
    recv_stream: RecvStream,
}

impl P2PBiStream {
    pub fn new(send_stream: SendStream, recv_stream: RecvStream) -> Self {
        Self { send_stream, recv_stream }
    }
}

impl tokio::io::AsyncRead for P2PBiStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.recv_stream).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for P2PBiStream {
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

impl RSocketConnection for P2PConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let bi_stream = P2PBiStream::new(self.send_stream, self.recv_stream);
        let (sink, stream) = Framed::new(bi_stream, LengthBasedFrameCodec).split();
        
        (
            Box::new(sink.sink_map_err(|e| RSocketError::Other(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Frame sink error: {}", e)
            ).into()))),
            Box::new(stream.map(|next| next.map_err(|e| RSocketError::Other(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Frame stream error: {}", e)
            ).into())))),
        )
    }
}
