use futures::{SinkExt, StreamExt};
use quinn::{RecvStream, SendStream};
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use tokio_util::codec::{FramedRead, FramedWrite};

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

impl Connection for QuinnConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let sink = FramedWrite::new(self.send_stream, LengthBasedFrameCodec);
        let stream = FramedRead::new(self.recv_stream, LengthBasedFrameCodec);
        
        (
            Box::new(sink.sink_map_err(|e| RSocketError::Other(e.into()))),
            Box::new(stream.map(|next| next.map_err(|e| RSocketError::Other(e.into())))),
        )
    }
}
