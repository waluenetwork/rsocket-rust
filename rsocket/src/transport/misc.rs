
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::sync::Arc;

use crate::frame::Frame;

#[derive(Debug, Clone)]
pub(crate) struct StreamID {
    inner: Arc<AtomicU32>,
}

impl StreamID {
    pub(crate) fn new(value: u32) -> StreamID {
        let inner = Arc::new(AtomicU32::new(value));
        StreamID { inner }
    }

    pub(crate) fn next(&self) -> u32 {
        let counter = self.inner.clone();
        counter.fetch_add(2, Ordering::Relaxed)
    }
}

impl From<u32> for StreamID {
    fn from(v: u32) -> StreamID {
        StreamID::new(v)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Counter {
    inner: Arc<AtomicI64>,
}

impl Counter {
    pub(crate) fn new(value: i64) -> Counter {
        Counter {
            inner: Arc::new(AtomicI64::new(value)),
        }
    }

    pub(crate) fn count_down(&self) -> i64 {
        self.inner.fetch_add(-1, Ordering::AcqRel) - 1
    }
}

#[inline]
pub(crate) fn debug_frame(snd: bool, f: &Frame) {
    if snd {
        debug!("===> SND: {:?}", f);
    } else {
        debug!("<=== RCV: {:?}", f);
    }
}

pub struct FrameBatcher {
    batch: crossbeam_deque::Worker<crate::frame::Frame>,
    batch_size: usize,
}

impl FrameBatcher {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch: crossbeam_deque::Worker::new_fifo(),
            batch_size,
        }
    }
    
    pub fn add_frame(&self, frame: crate::frame::Frame) {
        self.batch.push(frame);
    }
    
    pub fn process_batch<F>(&self, processor: F) 
    where 
        F: Fn(&[crate::frame::Frame]),
    {
        let mut frames = Vec::with_capacity(self.batch_size);
        while let Some(frame) = self.batch.pop() {
            frames.push(frame);
            if frames.len() >= self.batch_size {
                processor(&frames);
                frames.clear();
            }
        }
        if !frames.is_empty() {
            processor(&frames);
        }
    }
}
