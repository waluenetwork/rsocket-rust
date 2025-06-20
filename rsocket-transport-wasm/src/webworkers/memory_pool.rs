use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct MemoryPool {
    small_buffers: VecDeque<Vec<u8>>,
    medium_buffers: VecDeque<Vec<u8>>,
    large_buffers: VecDeque<Vec<u8>>,
    total_allocated: AtomicUsize,
    total_reused: AtomicUsize,
    max_pool_size: usize,
}

impl MemoryPool {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            small_buffers: VecDeque::new(),
            medium_buffers: VecDeque::new(),
            large_buffers: VecDeque::new(),
            total_allocated: AtomicUsize::new(0),
            total_reused: AtomicUsize::new(0),
            max_pool_size,
        }
    }
    
    pub fn get_buffer(&mut self, min_size: usize) -> Vec<u8> {
        let buffer = if min_size <= 1024 {
            Self::get_from_pool(&mut self.small_buffers, 1024)
        } else if min_size <= 65536 {
            Self::get_from_pool(&mut self.medium_buffers, 65536)
        } else {
            Self::get_from_pool(&mut self.large_buffers, min_size.next_power_of_two())
        };
        
        match buffer {
            Some(mut buf) => {
                buf.clear();
                buf.reserve(min_size);
                self.total_reused.fetch_add(1, Ordering::Relaxed);
                buf
            }
            None => {
                self.total_allocated.fetch_add(1, Ordering::Relaxed);
                Vec::with_capacity(min_size.next_power_of_two())
            }
        }
    }
    
    pub fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        if buffer.capacity() == 0 {
            return;
        }
        
        buffer.clear();
        
        if buffer.capacity() <= 1024 {
            if self.small_buffers.len() < self.max_pool_size {
                self.small_buffers.push_back(buffer);
            }
        } else if buffer.capacity() <= 65536 {
            if self.medium_buffers.len() < self.max_pool_size {
                self.medium_buffers.push_back(buffer);
            }
        } else {
            if self.large_buffers.len() < self.max_pool_size {
                self.large_buffers.push_back(buffer);
            }
        }
    }
    
    fn get_from_pool(pool: &mut VecDeque<Vec<u8>>, min_capacity: usize) -> Option<Vec<u8>> {
        for _ in 0..pool.len() {
            if let Some(buffer) = pool.pop_front() {
                if buffer.capacity() >= min_capacity {
                    return Some(buffer);
                } else {
                    pool.push_back(buffer);
                }
            }
        }
        None
    }
    
    pub fn get_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            small_pool_size: self.small_buffers.len(),
            medium_pool_size: self.medium_buffers.len(),
            large_pool_size: self.large_buffers.len(),
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_reused: self.total_reused.load(Ordering::Relaxed),
            reuse_rate: {
                let allocated = self.total_allocated.load(Ordering::Relaxed);
                let reused = self.total_reused.load(Ordering::Relaxed);
                if allocated + reused > 0 {
                    reused as f64 / (allocated + reused) as f64
                } else {
                    0.0
                }
            },
        }
    }
    
    pub fn clear(&mut self) {
        self.small_buffers.clear();
        self.medium_buffers.clear();
        self.large_buffers.clear();
        self.total_allocated.store(0, Ordering::Relaxed);
        self.total_reused.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub small_pool_size: usize,
    pub medium_pool_size: usize,
    pub large_pool_size: usize,
    pub total_allocated: usize,
    pub total_reused: usize,
    pub reuse_rate: f64,
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new(100)
    }
}

impl Default for MemoryPoolStats {
    fn default() -> Self {
        Self {
            small_pool_size: 0,
            medium_pool_size: 0,
            large_pool_size: 0,
            total_allocated: 0,
            total_reused: 0,
            reuse_rate: 0.0,
        }
    }
}
