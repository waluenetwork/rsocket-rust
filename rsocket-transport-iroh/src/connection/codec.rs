use std::io::{Error, ErrorKind};

use bytes::{Buf, BytesMut};
use rsocket_rust::frame::Frame;
use rsocket_rust::utils::{u24, Writeable};
use tokio_util::codec::{Decoder, Encoder};

pub struct LengthBasedFrameCodec;

const LEN_BYTES: usize = 3;

impl Decoder for LengthBasedFrameCodec {
    type Item = Frame;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let actual = buf.len();
        log::debug!("🔍 Codec decode: buffer length = {}", actual);
        
        if actual < LEN_BYTES {
            log::debug!("⏳ Codec decode: need more bytes for length header (have {}, need {})", actual, LEN_BYTES);
            return Ok(None);
        }
        
        let l = u24::read(buf).into();
        log::debug!("📏 Codec decode: frame length = {}", l);
        
        if actual < LEN_BYTES + l {
            log::debug!("⏳ Codec decode: need more bytes for frame (have {}, need {})", actual, LEN_BYTES + l);
            return Ok(None);
        }
        
        buf.advance(LEN_BYTES);
        let mut bb = buf.split_to(l);
        log::debug!("🔧 Codec decode: attempting to decode frame of {} bytes", l);
        
        match Frame::decode(&mut bb) {
            Ok(v) => {
                log::debug!("✅ Codec decode: successfully decoded frame type={:?}, stream_id={}", 
                          v.get_flag(), v.get_stream_id());
                Ok(Some(v))
            },
            Err(e) => {
                log::error!("❌ Codec decode: frame decode error: {:?}", e);
                Err(Error::from(ErrorKind::InvalidInput))
            }
        }
    }
}

impl Encoder<Frame> for LengthBasedFrameCodec {
    type Error = Error;
    fn encode(&mut self, item: Frame, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let l = item.len();
        log::debug!("📤 Codec encode: encoding frame type={:?}, stream_id={}, length={}", 
                  item.get_flag(), item.get_stream_id(), l);
        
        buf.reserve(LEN_BYTES + l);
        u24::from(l).write_to(buf);
        item.write_to(buf);
        
        log::debug!("✅ Codec encode: frame encoded successfully, total buffer size={}", buf.len());
        Ok(())
    }
}
