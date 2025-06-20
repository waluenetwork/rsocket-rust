
use wasm_bindgen::prelude::*;

use web_sys::{Worker, MessageEvent, ErrorEvent};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct RSocketWorker {
    worker: Worker,
    task_queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
    is_busy: Arc<Mutex<bool>>,
    worker_id: usize,
}

#[derive(Debug)]
pub struct WorkerPool {
    workers: Vec<RSocketWorker>,
    next_worker: usize,
    total_tasks: usize,
    completed_tasks: usize,
}

impl RSocketWorker {
    pub fn new(worker_id: usize) -> Result<Self, JsValue> {
        let worker = Worker::new("/rsocket-worker.js")?;
        
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let is_busy = Arc::new(Mutex::new(false));
        
        let task_queue_clone = Arc::clone(&task_queue);
        let is_busy_clone = Arc::clone(&is_busy);
        
        let onmessage_callback = Closure::wrap(Box::new(move |_event: MessageEvent| {
            if let Ok(mut is_busy) = is_busy_clone.lock() {
                *is_busy = false;
            }
            
            if let Ok(mut queue) = task_queue_clone.lock() {
                if let Some(_next_task) = queue.pop_front() {
                    log::debug!("Processing next queued task");
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        let onerror_callback = Closure::wrap(Box::new(move |event: ErrorEvent| {
            log::error!("Worker {} error: {:?}", worker_id, event.message());
        }) as Box<dyn FnMut(_)>);
        
        worker.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        
        Ok(RSocketWorker {
            worker,
            task_queue,
            is_busy,
            worker_id,
        })
    }
    
    pub fn process_frame(&self, frame: Vec<u8>) -> Result<(), JsValue> {
        let is_busy = self.is_busy.lock().map_err(|_| JsValue::from_str("Mutex lock failed"))?;
        if *is_busy {
            drop(is_busy);
            self.task_queue.lock().map_err(|_| JsValue::from_str("Mutex lock failed"))?.push_back(frame);
            return Ok(());
        }
        drop(is_busy);
        
        *self.is_busy.lock().map_err(|_| JsValue::from_str("Mutex lock failed"))? = true;
        
        let js_array = js_sys::Uint8Array::new_with_length(frame.len() as u32);
        js_array.copy_from(&frame);
        
        self.worker.post_message(&js_array.into())?;
        
        Ok(())
    }
    
    pub fn is_busy(&self) -> bool {
        self.is_busy.lock().map(|is_busy| *is_busy).unwrap_or(false)
    }
    
    pub fn queue_length(&self) -> usize {
        self.task_queue.lock().map(|queue| queue.len()).unwrap_or(0)
    }
}

impl WorkerPool {
    pub fn new(worker_count: usize) -> Result<Self, JsValue> {
        let mut workers = Vec::new();
        
        for i in 0..worker_count {
            let worker = RSocketWorker::new(i)?;
            workers.push(worker);
        }
        
        Ok(WorkerPool {
            workers,
            next_worker: 0,
            total_tasks: 0,
            completed_tasks: 0,
        })
    }
    
    pub fn process_frame(&mut self, frame: Vec<u8>) -> Result<(), JsValue> {
        let worker_index = self.find_least_busy_worker();
        
        self.workers[worker_index].process_frame(frame)?;
        self.total_tasks += 1;
        
        Ok(())
    }
    
    pub fn process_batch(&mut self, frames: Vec<Vec<u8>>) -> Result<(), JsValue> {
        for frame in frames {
            self.process_frame(frame)?;
        }
        Ok(())
    }
    
    fn find_least_busy_worker(&mut self) -> usize {
        let mut best_worker = 0;
        let mut min_queue_length = usize::MAX;
        
        for (i, worker) in self.workers.iter().enumerate() {
            let queue_length = worker.queue_length();
            if queue_length < min_queue_length {
                min_queue_length = queue_length;
                best_worker = i;
            }
        }
        
        best_worker
    }
    
    pub fn get_worker_utilization(&self) -> f64 {
        let busy_workers = self.workers.iter()
            .filter(|w| w.is_busy())
            .count();
        
        busy_workers as f64 / self.workers.len() as f64
    }
    
    pub fn get_total_queue_length(&self) -> usize {
        self.workers.iter()
            .map(|w| w.queue_length())
            .sum()
    }
}

pub fn is_webworkers_supported() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::eval("typeof Worker !== 'undefined'")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}
