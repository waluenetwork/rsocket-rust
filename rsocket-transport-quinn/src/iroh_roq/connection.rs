use std::pin::Pin;
use std::task::{Context, Poll};
use futures::{Sink, Stream};
use rsocket_rust::error::RSocketError;
use rsocket_rust::frame::Frame;
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use rsocket_rust::utils::Writeable;
use iroh::endpoint::VarInt;
use rtp::packet::Packet as RtpPacket;
use bytes::{Bytes, BytesMut};

use super::session::{IrohRoqSession, IrohRoqSessionConfig};

#[derive(Debug)]
pub struct IrohRoqConnection {
    session: IrohRoqSession,
    flow_id: VarInt,
}

impl IrohRoqConnection {
    pub fn new(session: IrohRoqSession, flow_id: VarInt) -> Self {
        Self { session, flow_id }
    }

    pub fn from_quic_connection(conn: iroh::endpoint::Connection) -> Self {
        let config = IrohRoqSessionConfig::default();
        let session = IrohRoqSession::new(conn, config);
        let flow_id = VarInt::from_u32(0);
        Self::new(session, flow_id)
    }
}

impl Connection for IrohRoqConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let sink = IrohRoqSink::new(self.session.clone(), self.flow_id);
        let stream = IrohRoqStream::new(self.session, self.flow_id);
        
        (Box::new(sink), Box::new(stream))
    }
}

struct IrohRoqSink {
    session: IrohRoqSession,
    flow_id: VarInt,
}

impl IrohRoqSink {
    fn new(session: IrohRoqSession, flow_id: VarInt) -> Self {
        Self { session, flow_id }
    }

    fn frame_to_rtp(&self, frame: Frame) -> Result<RtpPacket, RSocketError> {
        let mut buf = BytesMut::new();
        frame.write_to(&mut buf);
        let payload = buf.freeze();
        
        Ok(RtpPacket {
            header: rtp::header::Header {
                version: 2,
                padding: false,
                extension: false,
                extension_profile: 0,
                marker: false,
                payload_type: 96,
                sequence_number: 0,
                timestamp: 0,
                ssrc: self.flow_id.into_inner() as u32,
                csrc: vec![],
                extensions: vec![],
                extensions_padding: 0,
            },
            payload: payload.to_vec().into(),
        })
    }
}

impl Sink<Frame> for IrohRoqSink {
    type Error = RSocketError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Frame) -> Result<(), Self::Error> {
        let rtp_packet = self.frame_to_rtp(item)?;
        
        let session = self.session.clone();
        tokio::spawn(async move {
            if let Err(e) = session.send_rtp_packet(&rtp_packet).await {
                log::error!("Failed to send RTP packet: {:?}", e);
            }
        });
        
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

struct IrohRoqStream {
    session: IrohRoqSession,
    flow_id: VarInt,
}

impl IrohRoqStream {
    fn new(session: IrohRoqSession, flow_id: VarInt) -> Self {
        Self { session, flow_id }
    }

    fn rtp_to_frame(&self, packet: RtpPacket) -> Result<Frame, RSocketError> {
        let payload_bytes = Bytes::from(packet.payload.to_vec());
        let mut buf = BytesMut::from(payload_bytes.as_ref());
        Frame::decode(&mut buf).map_err(|e| RSocketError::Other(e.into()))
    }
}

impl Stream for IrohRoqStream {
    type Item = Result<Frame, RSocketError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}
