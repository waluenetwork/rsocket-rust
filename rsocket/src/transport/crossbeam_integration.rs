use std::sync::Arc;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossbeam_skiplist::SkipMap;
use crossbeam_deque::{Injector, Stealer, Worker};

use super::socket::{DuplexSocket, DuplexSocketInner};
use dashmap::DashMap;
use super::misc::{debug_frame, StreamID};
use crate::error::{self, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::payload::{Payload, SetupPayload};
use crate::spi::{Flux, RSocket, ServerResponder};
use crate::Result;

use futures::future::{AbortHandle, Abortable};
use tokio::sync::{oneshot, RwLock};

pub struct CrossbeamOptimizedSocket {
    inner: Arc<DuplexSocketInner>,
    frame_processor: Arc<super::simd_frame_processing::SimdFrameProcessor>,
    frame_batcher: Arc<super::misc::FrameBatcher>,
}

impl CrossbeamOptimizedSocket {
    pub fn new(
        first_stream_id: u32,
        tx: tokio::sync::mpsc::UnboundedSender<Frame>,
        splitter: Option<super::fragmentation::Splitter>,
    ) -> Self {
        let socket = DuplexSocket::new(first_stream_id, tx, splitter);
        let frame_processor = Arc::new(super::simd_frame_processing::SimdFrameProcessor::new());
        let frame_batcher = Arc::new(super::misc::FrameBatcher::new(64));
        
        Self {
            inner: socket.inner,
            frame_processor,
            frame_batcher,
        }
    }
    
    pub fn process_frames_optimized(&self, frames: &[Frame]) -> Vec<bytes::Bytes> {
        self.frame_processor.batch_process_frames(frames)
    }
    
    pub fn add_frame_to_batch(&self, frame: Frame) {
        self.frame_batcher.add_frame(frame);
    }
    
    pub fn process_batched_frames<F>(&self, processor: F)
    where
        F: Fn(&[Frame]),
    {
        self.frame_batcher.process_batch(processor);
    }
}

impl From<DuplexSocket> for CrossbeamOptimizedSocket {
    fn from(socket: DuplexSocket) -> Self {
        let frame_processor = Arc::new(super::simd_frame_processing::SimdFrameProcessor::new());
        let frame_batcher = Arc::new(super::misc::FrameBatcher::new(64));
        
        Self {
            inner: socket.inner,
            frame_processor,
            frame_batcher,
        }
    }
}
