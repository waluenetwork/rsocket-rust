#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use bytes::{Bytes, BytesMut};
use crate::frame::Frame;
use crate::utils::Writeable;

pub struct SimdFrameProcessor {
    buffer_pool: crossbeam_deque::Injector<BytesMut>,
}

impl SimdFrameProcessor {
    pub fn new() -> Self {
        let buffer_pool = crossbeam_deque::Injector::new();
        for _ in 0..1024 {
            buffer_pool.push(BytesMut::with_capacity(4096));
        }
        
        Self { buffer_pool }
    }
    
    #[cfg(target_arch = "x86_64")]
    pub fn batch_process_frames(&self, frames: &[Frame]) -> Vec<Bytes> {
        let mut results = Vec::with_capacity(frames.len());
        
        for chunk in frames.chunks(8) {
            for frame in chunk {
                let serialized = unsafe { self.simd_serialize_frame(frame) };
                results.push(serialized);
            }
        }
        
        results
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe fn simd_serialize_frame(&self, frame: &Frame) -> Bytes {
        let mut buffer = match self.buffer_pool.steal() {
            crossbeam_deque::Steal::Success(buf) => buf,
            _ => BytesMut::with_capacity(4096),
        };
        
        let header_data = [
            frame.get_stream_id() as u8,
            (frame.get_stream_id() >> 8) as u8,
            (frame.get_stream_id() >> 16) as u8,
            (frame.get_stream_id() >> 24) as u8,
            frame.get_flag() as u8,
            (frame.get_flag() >> 8) as u8,
            0, 0,
        ];
        
        let header_simd = _mm_loadu_si128(header_data.as_ptr() as *const __m128i);
        let mut header_bytes = [0u8; 16];
        _mm_storeu_si128(header_bytes.as_mut_ptr() as *mut __m128i, header_simd);
        
        buffer.extend_from_slice(&header_bytes[..6]);
        
        frame.write_to(&mut buffer);
        
        let result = buffer.freeze();
        
        buffer = BytesMut::with_capacity(4096);
        self.buffer_pool.push(buffer);
        
        result
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    pub fn batch_process_frames(&self, frames: &[Frame]) -> Vec<Bytes> {
        frames.iter().map(|frame| {
            let mut buffer = BytesMut::new();
            frame.write_to(&mut buffer);
            buffer.freeze()
        }).collect()
    }
}

impl Default for SimdFrameProcessor {
    fn default() -> Self {
        Self::new()
    }
}
