
use wasm_bindgen::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct SharedRingBuffer {
    buffer: js_sys::SharedArrayBuffer,
    capacity: usize,
    read_pos: AtomicUsize,
    write_pos: AtomicUsize,
    frame_count: AtomicUsize,
}

impl SharedRingBuffer {
    pub fn new(capacity: usize) -> Result<Self, JsValue> {
        let buffer = js_sys::SharedArrayBuffer::new(capacity as u32);
        
        Ok(SharedRingBuffer {
            buffer,
            capacity,
            read_pos: AtomicUsize::new(0),
            write_pos: AtomicUsize::new(0),
            frame_count: AtomicUsize::new(0),
        })
    }
    
    pub fn write_frame(&self, frame: &[u8]) -> Result<bool, JsValue> {
        let frame_len = frame.len();
        
        let required_space = frame_len + 4;
        if required_space > self.available_write_space() {
            return Ok(false); // Buffer full
        }
        
        let write_pos = self.write_pos.load(Ordering::Acquire);
        
        let view = js_sys::Uint8Array::new_with_byte_offset_and_length(
            &self.buffer,
            write_pos as u32,
            required_space as u32,
        );
        
        let len_bytes = (frame_len as u32).to_le_bytes();
        for (i, &byte) in len_bytes.iter().enumerate() {
            view.set_index(i as u32, byte);
        }
        
        for (i, &byte) in frame.iter().enumerate() {
            view.set_index((i + 4) as u32, byte);
        }
        
        let new_write_pos = (write_pos + required_space) % self.capacity;
        self.write_pos.store(new_write_pos, Ordering::Release);
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(true)
    }
    
    pub fn read_frame(&self) -> Result<Option<Vec<u8>>, JsValue> {
        if self.frame_count.load(Ordering::Relaxed) == 0 {
            return Ok(None);
        }
        
        let read_pos = self.read_pos.load(Ordering::Acquire);
        
        let len_view = js_sys::Uint8Array::new_with_byte_offset_and_length(
            &self.buffer,
            read_pos as u32,
            4,
        );
        
        let mut len_bytes = [0u8; 4];
        for i in 0..4 {
            len_bytes[i] = len_view.get_index(i as u32);
        }
        let frame_len = u32::from_le_bytes(len_bytes) as usize;
        
        let data_view = js_sys::Uint8Array::new_with_byte_offset_and_length(
            &self.buffer,
            (read_pos + 4) as u32,
            frame_len as u32,
        );
        
        let mut frame = vec![0u8; frame_len];
        for i in 0..frame_len {
            frame[i] = data_view.get_index(i as u32);
        }
        
        let new_read_pos = (read_pos + frame_len + 4) % self.capacity;
        self.read_pos.store(new_read_pos, Ordering::Release);
        self.frame_count.fetch_sub(1, Ordering::Relaxed);
        
        Ok(Some(frame))
    }
    
    pub fn available_write_space(&self) -> usize {
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        
        if write_pos >= read_pos {
            self.capacity - write_pos + read_pos - 1
        } else {
            read_pos - write_pos - 1
        }
    }
    
    pub fn frame_count(&self) -> usize {
        self.frame_count.load(Ordering::Relaxed)
    }
    
    pub fn is_empty(&self) -> bool {
        self.frame_count() == 0
    }
    
    pub fn is_full(&self) -> bool {
        self.available_write_space() < 8 // Minimum space for a frame (4 bytes length + 4 bytes data)
    }
    
    pub fn get_shared_buffer(&self) -> &js_sys::SharedArrayBuffer {
        &self.buffer
    }
}

pub fn is_shared_array_buffer_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shared_ring_buffer_basic() {
        if !is_shared_array_buffer_supported() {
            return;
        }
        
        let buffer = SharedRingBuffer::new(1024).unwrap();
        
        let test_frame = vec![1, 2, 3, 4, 5];
        assert!(buffer.write_frame(&test_frame).unwrap());
        
        let read_frame = buffer.read_frame().unwrap().unwrap();
        assert_eq!(read_frame, test_frame);
        
        assert!(buffer.read_frame().unwrap().is_none());
    }
}
