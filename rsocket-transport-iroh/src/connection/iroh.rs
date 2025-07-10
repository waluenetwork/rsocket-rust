use futures::{SinkExt, StreamExt};
use iroh::endpoint::Connection;
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection as RSocketConnection, FrameSink, FrameStream};
use tokio_util::codec::Framed;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::codec::LengthBasedFrameCodec;

#[derive(Debug)]
pub struct IrohConnection {
    connection: Connection,
}

impl IrohConnection {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

#[derive(Debug)]
pub struct IrohBiStream {
    send_stream: iroh::endpoint::SendStream,
    recv_stream: iroh::endpoint::RecvStream,
}

impl IrohBiStream {
    pub fn new(send_stream: iroh::endpoint::SendStream, recv_stream: iroh::endpoint::RecvStream) -> Self {
        Self { send_stream, recv_stream }
    }
}

impl tokio::io::AsyncRead for IrohBiStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before_len = buf.filled().len();
        match Pin::new(&mut self.recv_stream).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                let bytes_read = buf.filled().len() - before_len;
                if bytes_read > 0 {
                    log::debug!("✅ IrohBiStream: Read {} bytes from recv stream", bytes_read);
                } else {
                    log::debug!("✅ IrohBiStream: Read completed (EOF)");
                }
                Poll::Ready(Ok(()))
            },
            Poll::Ready(Err(e)) => {
                log::error!("❌ IrohBiStream: Read error: {:?}", e);
                Poll::Ready(Err(e))
            },
            Poll::Pending => {
                log::debug!("⏳ IrohBiStream: Read pending");
                Poll::Pending
            }
        }
    }
}

impl tokio::io::AsyncWrite for IrohBiStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match Pin::new(&mut self.send_stream).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => {
                log::debug!("✅ IrohBiStream: Wrote {} bytes to send stream", n);
                Poll::Ready(Ok(n))
            },
            Poll::Ready(Err(e)) => {
                log::error!("❌ IrohBiStream: Write error: {:?}", e);
                Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, format!("Iroh send stream error: {}", e))))
            },
            Poll::Pending => {
                log::debug!("⏳ IrohBiStream: Write pending");
                Poll::Pending
            }
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match Pin::new(&mut self.send_stream).poll_flush(cx) {
            Poll::Ready(Ok(())) => {
                log::debug!("✅ IrohBiStream: Flush completed");
                Poll::Ready(Ok(()))
            },
            Poll::Ready(Err(e)) => {
                log::error!("❌ IrohBiStream: Flush error: {:?}", e);
                Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, format!("Iroh flush error: {}", e))))
            },
            Poll::Pending => {
                log::debug!("⏳ IrohBiStream: Flush pending");
                Poll::Pending
            }
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match Pin::new(&mut self.send_stream).poll_shutdown(cx) {
            Poll::Ready(Ok(())) => {
                log::debug!("✅ IrohBiStream: Shutdown completed");
                Poll::Ready(Ok(()))
            },
            Poll::Ready(Err(e)) => {
                log::error!("❌ IrohBiStream: Shutdown error: {:?}", e);
                Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, format!("Iroh shutdown error: {}", e))))
            },
            Poll::Pending => {
                log::debug!("⏳ IrohBiStream: Shutdown pending");
                Poll::Pending
            }
        }
    }
}

impl RSocketConnection for IrohConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let connection = self.connection;
        let rt = tokio::runtime::Handle::current();
        
        let (send_stream, recv_stream) = rt.block_on(async {
            connection.open_bi().await.map_err(|e| {
                log::error!("❌ Failed to open bidirectional stream: {:?}", e);
                e
            })
        }).expect("Failed to open bidirectional stream");
        
        log::info!("✅ Opened bidirectional stream for RSocket frames");
        
        let bi_stream = IrohBiStream::new(send_stream, recv_stream);
        let (sink, stream) = Framed::new(bi_stream, LengthBasedFrameCodec).split();
        
        let enhanced_sink = sink.sink_map_err(|e| {
            log::error!("❌ RSocket frame sink error: {:?}", e);
            RSocketError::Other(e.into())
        });
        
        let enhanced_stream = stream.map(|next| {
            match &next {
                Ok(frame) => {
                    log::debug!("✅ RSocket frame received: type={:?}, stream_id={}", 
                              frame.get_flag(), frame.get_stream_id());
                },
                Err(e) => {
                    log::error!("❌ RSocket frame decode error: {:?}", e);
                }
            }
            next.map_err(|e| RSocketError::Other(e.into()))
        });
        
        (
            Box::new(enhanced_sink),
            Box::new(enhanced_stream),
        )
    }
}

#[derive(Debug)]
pub struct IrohConnectionWithStreams {
    bi_stream: IrohBiStream,
}

impl IrohConnectionWithStreams {
    pub fn new(send_stream: iroh::endpoint::SendStream, recv_stream: iroh::endpoint::RecvStream) -> Self {
        Self {
            bi_stream: IrohBiStream::new(send_stream, recv_stream),
        }
    }
}

impl RSocketConnection for IrohConnectionWithStreams {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        log::info!("✅ Splitting pre-opened Iroh bidirectional stream for RSocket frames");
        
        let (sink, stream) = Framed::new(self.bi_stream, LengthBasedFrameCodec).split();
        
        let enhanced_sink = sink.sink_map_err(|e| {
            log::error!("❌ RSocket frame sink error: {:?}", e);
            RSocketError::Other(e.into())
        });
        
        let enhanced_stream = stream.map(|next| {
            match &next {
                Ok(frame) => {
                    log::debug!("✅ RSocket frame received: type={:?}, stream_id={}", 
                              frame.get_flag(), frame.get_stream_id());
                },
                Err(e) => {
                    log::error!("❌ RSocket frame decode error: {:?}", e);
                }
            }
            next.map_err(|e| RSocketError::Other(e.into()))
        });
        
        (
            Box::new(enhanced_sink),
            Box::new(enhanced_stream),
        )
    }
}
