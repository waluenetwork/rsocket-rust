
use wasm_bindgen::prelude::*;
use js_sys::{SharedArrayBuffer, Uint8Array};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub size: usize,
    pub max_frame_size: usize,
    pub slot_count: usize,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            size: 1024 * 1024, // 1MB
            max_frame_size: 64 * 1024, // 64KB
            slot_count: 256,
        }
    }
}

#[derive(Debug)]
pub struct SharedRingBuffer {
    buffer: SharedArrayBuffer,
    config: BufferConfig,
    read_pos: Arc<AtomicUsize>,
    write_pos: Arc<AtomicUsize>,
    available_slots: Arc<AtomicUsize>,
}

impl SharedRingBuffer {
    pub fn new(config: BufferConfig) -> Result<Self, JsValue> {
        let buffer = SharedArrayBuffer::new(config.size as u32);
        
        let slot_count = config.slot_count;
        Ok(Self {
            buffer,
            config,
            read_pos: Arc::new(AtomicUsize::new(0)),
            write_pos: Arc::new(AtomicUsize::new(0)),
            available_slots: Arc::new(AtomicUsize::new(slot_count)),
        })
    }

    pub fn try_write(&self, data: &[u8]) -> Result<bool, JsValue> {
        if data.len() > self.config.max_frame_size {
            return Ok(false); // Frame too large
        }

        let available = self.available_slots.load(Ordering::Acquire);
        if available == 0 {
            return Ok(false); // Buffer full
        }

        let write_pos = self.write_pos.load(Ordering::Acquire);
        let slot_size = self.config.size / self.config.slot_count;
        let slot_offset = (write_pos % self.config.slot_count) * slot_size;

        let length_bytes = (data.len() as u32).to_le_bytes();
        let buffer_view = Uint8Array::new(&self.buffer);
        
        for (i, &byte) in length_bytes.iter().enumerate() {
            buffer_view.set_index(slot_offset as u32 + i as u32, byte);
        }

        for (i, &byte) in data.iter().enumerate() {
            buffer_view.set_index(slot_offset as u32 + 4 + i as u32, byte);
        }

        self.write_pos.store((write_pos + 1) % self.config.slot_count, Ordering::Release);
        self.available_slots.fetch_sub(1, Ordering::AcqRel);

        Ok(true)
    }

    pub fn try_read(&self) -> Result<Option<Vec<u8>>, JsValue> {
        let available = self.available_slots.load(Ordering::Acquire);
        if available == self.config.slot_count {
            return Ok(None); // Buffer empty
        }

        let read_pos = self.read_pos.load(Ordering::Acquire);
        let slot_size = self.config.size / self.config.slot_count;
        let slot_offset = (read_pos % self.config.slot_count) * slot_size;

        let buffer_view = Uint8Array::new(&self.buffer);

        let mut length_bytes = [0u8; 4];
        for i in 0..4 {
            length_bytes[i] = buffer_view.get_index(slot_offset as u32 + i as u32);
        }
        let frame_length = u32::from_le_bytes(length_bytes) as usize;

        if frame_length > self.config.max_frame_size {
            return Err(JsValue::from_str("Invalid frame length"));
        }

        let mut frame_data = vec![0u8; frame_length];
        for i in 0..frame_length {
            frame_data[i] = buffer_view.get_index(slot_offset as u32 + 4 + i as u32);
        }

        self.read_pos.store((read_pos + 1) % self.config.slot_count, Ordering::Release);
        self.available_slots.fetch_add(1, Ordering::AcqRel);

        Ok(Some(frame_data))
    }

    pub fn get_shared_buffer(&self) -> &SharedArrayBuffer {
        &self.buffer
    }

    pub fn get_utilization(&self) -> f64 {
        let available = self.available_slots.load(Ordering::Acquire);
        let used = self.config.slot_count - available;
        used as f64 / self.config.slot_count as f64
    }

    pub fn is_nearly_full(&self) -> bool {
        self.get_utilization() > 0.8
    }

    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            total_slots: self.config.slot_count,
            available_slots: self.available_slots.load(Ordering::Acquire),
            utilization: self.get_utilization(),
            read_pos: self.read_pos.load(Ordering::Acquire),
            write_pos: self.write_pos.load(Ordering::Acquire),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferStats {
    pub total_slots: usize,
    pub available_slots: usize,
    pub utilization: f64,
    pub read_pos: usize,
    pub write_pos: usize,
}

#[derive(Debug)]
pub struct FallbackRingBuffer {
    buffer: Vec<Option<Vec<u8>>>,
    config: BufferConfig,
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl FallbackRingBuffer {
    pub fn new(config: BufferConfig) -> Self {
        Self {
            buffer: vec![None; config.slot_count],
            config,
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    pub fn try_write(&mut self, data: &[u8]) -> bool {
        if self.count >= self.config.slot_count || data.len() > self.config.max_frame_size {
            return false;
        }

        self.buffer[self.write_pos] = Some(data.to_vec());
        self.write_pos = (self.write_pos + 1) % self.config.slot_count;
        self.count += 1;
        true
    }

    pub fn try_read(&mut self) -> Option<Vec<u8>> {
        if self.count == 0 {
            return None;
        }

        let data = self.buffer[self.read_pos].take();
        self.read_pos = (self.read_pos + 1) % self.config.slot_count;
        self.count -= 1;
        data
    }

    pub fn get_utilization(&self) -> f64 {
        self.count as f64 / self.config.slot_count as f64
    }
}

pub fn create_ring_buffer(config: BufferConfig) -> Result<Box<dyn RingBuffer>, JsValue> {
    if is_shared_array_buffer_supported() {
        Ok(Box::new(SharedRingBuffer::new(config)?))
    } else {
        Ok(Box::new(FallbackRingBuffer::new(config)))
    }
}

pub trait RingBuffer {
    fn try_write(&mut self, data: &[u8]) -> bool;
    fn try_read(&mut self) -> Option<Vec<u8>>;
    fn get_utilization(&self) -> f64;
}

impl RingBuffer for SharedRingBuffer {
    fn try_write(&mut self, data: &[u8]) -> bool {
        match SharedRingBuffer::try_write(self, data) {
            Ok(result) => result,
            Err(_) => false,
        }
    }

    fn try_read(&mut self) -> Option<Vec<u8>> {
        match SharedRingBuffer::try_read(self) {
            Ok(result) => result,
            Err(_) => None,
        }
    }

    fn get_utilization(&self) -> f64 {
        SharedRingBuffer::get_utilization(self)
    }
}

impl RingBuffer for FallbackRingBuffer {
    fn try_write(&mut self, data: &[u8]) -> bool {
        self.try_write(data)
    }

    fn try_read(&mut self) -> Option<Vec<u8>> {
        self.try_read()
    }

    fn get_utilization(&self) -> f64 {
        self.get_utilization()
    }
}

fn is_shared_array_buffer_supported() -> bool {
    use wasm_bindgen::JsValue;
    use js_sys::global;
    
    let global = global();
    js_sys::Reflect::has(&global, &JsValue::from_str("SharedArrayBuffer"))
        .unwrap_or(false)
}
