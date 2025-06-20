
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{WebSocket, MessageEvent, Worker};
use js_sys::{Uint8Array, ArrayBuffer};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use super::WebWorkersConfig;

#[derive(Debug, Clone)]
pub struct SimpleWasmPerformanceMetrics {
    pub messages_per_second: f64,
    pub average_latency: f64,
    pub total_messages: u32,
    pub total_bytes: usize,
    pub worker_utilization: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    messages_sent: u32,
    total_bytes: usize,
    start_time: f64,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            total_bytes: 0,
            start_time: js_sys::Date::now(),
        }
    }

    pub fn record_message(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.total_bytes += bytes;
    }

    pub fn record_latency(&mut self, _latency: f64) {
    }

    pub fn record_worker_response(&mut self, _worker_id: usize) {
    }

    pub fn get_metrics(&self) -> SimpleWasmPerformanceMetrics {
        let elapsed = js_sys::Date::now() - self.start_time;
        SimpleWasmPerformanceMetrics {
            messages_per_second: if elapsed > 0.0 { self.messages_sent as f64 / (elapsed / 1000.0) } else { 0.0 },
            average_latency: 0.0,
            total_messages: self.messages_sent,
            total_bytes: self.total_bytes,
            worker_utilization: 0.0,
        }
    }

    pub fn log_performance_summary(&self) {
        let metrics = self.get_metrics();
        web_sys::console::log_1(&format!(
            "Performance: {:.0} msg/sec, {} total messages, {} total bytes",
            metrics.messages_per_second, metrics.total_messages, metrics.total_bytes
        ).into());
    }
}

#[derive(Debug)]
pub struct SimpleWasmWebWorkersClient {
    websocket_url: String,
    config: WebWorkersConfig,
    workers: Vec<Worker>,
    performance_monitor: Option<PerformanceMonitor>,
    message_id_counter: Rc<RefCell<u32>>,
    pending_responses: Rc<RefCell<HashMap<u32, js_sys::Function>>>,
}

#[derive(Debug)]
pub struct SimpleWasmConnection {
    websocket: WebSocket,
    workers: Vec<Worker>,
    config: WebWorkersConfig,
    performance_monitor: Option<PerformanceMonitor>,
    message_id_counter: Rc<RefCell<u32>>,
    pending_responses: Rc<RefCell<HashMap<u32, js_sys::Function>>>,
}

impl SimpleWasmWebWorkersClient {
    pub fn new(websocket_url: String, config: WebWorkersConfig) -> Self {
        let performance_monitor = if config.enable_performance_monitoring {
            Some(PerformanceMonitor::new())
        } else {
            None
        };

        Self {
            websocket_url,
            config,
            workers: Vec::new(),
            performance_monitor,
            message_id_counter: Rc::new(RefCell::new(1)),
            pending_responses: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub async fn connect(&mut self) -> Result<SimpleWasmConnection, JsValue> {
        let websocket = WebSocket::new(&self.websocket_url)?;
        websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);

        self.initialize_workers().await?;

        let connection = SimpleWasmConnection {
            websocket,
            workers: self.workers.clone(),
            config: self.config.clone(),
            performance_monitor: self.performance_monitor.take(),
            message_id_counter: Rc::clone(&self.message_id_counter),
            pending_responses: Rc::clone(&self.pending_responses),
        };

        Ok(connection)
    }

    async fn initialize_workers(&mut self) -> Result<(), JsValue> {
        if !self.workers.is_empty() {
            return Ok(());
        }

        for i in 0..self.config.worker_count {
            let worker_script = self.create_worker_script();
            let blob = web_sys::Blob::new_with_str_sequence_and_options(
                &js_sys::Array::of1(&worker_script.into()),
                web_sys::BlobPropertyBag::new().type_("application/javascript"),
            )?;

            let worker_url = web_sys::Url::create_object_url_with_blob(&blob)?;
            let worker = Worker::new(&worker_url)?;

            let pending_responses = Rc::clone(&self.pending_responses);

            let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(_data) = event.data().dyn_into::<js_sys::Object>() {
                    web_sys::console::log_1(&format!("Worker {} response received", i).into());
                }
            }) as Box<dyn FnMut(_)>);

            worker.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();

            self.workers.push(worker);
        }

        Ok(())
    }

    fn create_worker_script(&self) -> String {
        r#"
        self.onmessage = function(event) {
            const { messageId, frameData, operation } = event.data;
            
            try {
                let result;
                switch (operation) {
                    case 'process_frame':
                        result = processFrame(frameData);
                        break;
                    case 'batch_process':
                        result = batchProcessFrames(frameData);
                        break;
                    default:
                        result = { error: 'Unknown operation' };
                }
                
                self.postMessage({
                    messageId: messageId,
                    result: result,
                    timestamp: performance.now()
                });
            } catch (error) {
                self.postMessage({
                    messageId: messageId,
                    error: error.message,
                    timestamp: performance.now()
                });
            }
        };
        
        function processFrame(frameData) {
            const processed = new Uint8Array(frameData.length);
            for (let i = 0; i < frameData.length; i++) {
                processed[i] = frameData[i] ^ 0x42; // Simple XOR processing
            }
            return processed;
        }
        
        function batchProcessFrames(frames) {
            return frames.map(frame => processFrame(frame));
        }
        "#.to_string()
    }

    pub fn get_performance_metrics(&self) -> Option<SimpleWasmPerformanceMetrics> {
        self.performance_monitor.as_ref().map(|m| m.get_metrics())
    }

    pub fn is_supported() -> bool {
        js_sys::eval("typeof Worker !== 'undefined'").unwrap_or(JsValue::FALSE).is_truthy()
    }
}

impl SimpleWasmConnection {
    pub async fn send_frame(&mut self, frame_data: Vec<u8>) -> Result<(), JsValue> {
        let start_time = js_sys::Date::now();

        if let Some(monitor) = &mut self.performance_monitor {
            monitor.record_message(frame_data.len());
        }

        if !self.workers.is_empty() {
            self.process_frame_with_workers(frame_data.clone()).await?;
        }

        let array = Uint8Array::new_with_length(frame_data.len() as u32);
        array.copy_from(&frame_data);
        self.websocket.send_with_array_buffer(&array.buffer())?;

        if let Some(monitor) = &mut self.performance_monitor {
            let latency = js_sys::Date::now() - start_time;
            monitor.record_latency(latency);
        }

        Ok(())
    }

    pub async fn send_frame_batch(&mut self, frames: Vec<Vec<u8>>) -> Result<(), JsValue> {
        if frames.is_empty() {
            return Ok(());
        }

        let start_time = js_sys::Date::now();

        if let Some(monitor) = &mut self.performance_monitor {
            for frame in &frames {
                monitor.record_message(frame.len());
            }
        }

        if !self.workers.is_empty() {
            self.process_frame_batch_with_workers(frames.clone()).await?;
        }

        for frame_data in frames {
            let array = Uint8Array::new_with_length(frame_data.len() as u32);
            array.copy_from(&frame_data);
            self.websocket.send_with_array_buffer(&array.buffer())?;
        }

        if let Some(monitor) = &mut self.performance_monitor {
            let latency = js_sys::Date::now() - start_time;
            monitor.record_latency(latency);
        }

        Ok(())
    }

    async fn process_frame_with_workers(&mut self, frame_data: Vec<u8>) -> Result<(), JsValue> {
        if self.workers.is_empty() {
            return Ok(());
        }

        let worker_index = self.get_next_worker_index();
        let worker = &self.workers[worker_index];
        let message_id = self.get_next_message_id();

        let message = js_sys::Object::new();
        js_sys::Reflect::set(&message, &"messageId".into(), &message_id.into())?;
        js_sys::Reflect::set(&message, &"operation".into(), &"process_frame".into())?;
        
        let frame_array = Uint8Array::new_with_length(frame_data.len() as u32);
        frame_array.copy_from(&frame_data);
        js_sys::Reflect::set(&message, &"frameData".into(), &frame_array)?;

        worker.post_message(&message)?;
        Ok(())
    }

    async fn process_frame_batch_with_workers(&mut self, frames: Vec<Vec<u8>>) -> Result<(), JsValue> {
        if self.workers.is_empty() {
            return Ok(());
        }

        let worker_index = self.get_next_worker_index();
        let worker = &self.workers[worker_index];
        let message_id = self.get_next_message_id();

        let message = js_sys::Object::new();
        js_sys::Reflect::set(&message, &"messageId".into(), &message_id.into())?;
        js_sys::Reflect::set(&message, &"operation".into(), &"batch_process".into())?;

        let frames_array = js_sys::Array::new();
        for frame_data in frames {
            let frame_array = Uint8Array::new_with_length(frame_data.len() as u32);
            frame_array.copy_from(&frame_data);
            frames_array.push(&frame_array);
        }
        js_sys::Reflect::set(&message, &"frameData".into(), &frames_array)?;

        worker.post_message(&message)?;
        Ok(())
    }

    fn get_next_worker_index(&self) -> usize {
        let message_id = *self.message_id_counter.borrow();
        (message_id as usize) % self.workers.len()
    }

    fn get_next_message_id(&self) -> u32 {
        let mut counter = self.message_id_counter.borrow_mut();
        let id = *counter;
        *counter = counter.wrapping_add(1);
        id
    }

    pub fn get_worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn get_performance_metrics(&self) -> Option<SimpleWasmPerformanceMetrics> {
        self.performance_monitor.as_ref().map(|m| m.get_metrics())
    }

    pub fn log_performance_summary(&self) {
        if let Some(monitor) = &self.performance_monitor {
            monitor.log_performance_summary();
        }
    }
}

impl From<String> for SimpleWasmWebWorkersClient {
    fn from(websocket_url: String) -> Self {
        Self::new(websocket_url, WebWorkersConfig::default())
    }
}

impl From<&str> for SimpleWasmWebWorkersClient {
    fn from(websocket_url: &str) -> Self {
        Self::from(websocket_url.to_string())
    }
}

pub fn create_simple_wasm_config() -> WebWorkersConfig {
    let worker_count = js_sys::eval("navigator.hardwareConcurrency || 4")
        .unwrap_or(JsValue::from(4))
        .as_f64()
        .unwrap_or(4.0) as usize;
    
    let shared_array_buffer_supported = js_sys::eval("typeof SharedArrayBuffer !== 'undefined'")
        .unwrap_or(JsValue::FALSE)
        .is_truthy();
    
    WebWorkersConfig {
        worker_count,
        buffer_size: if shared_array_buffer_supported { 
            2 * 1024 * 1024 // 2MB with SharedArrayBuffer
        } else { 
            512 * 1024 // 512KB fallback
        },
        batch_size: 100,
        enable_shared_array_buffer: shared_array_buffer_supported,
        enable_performance_monitoring: true,
        enable_zero_copy: shared_array_buffer_supported,
        max_concurrent_tasks: 1000,
    }
}

pub async fn benchmark_simple_wasm_performance(
    websocket_url: String,
    duration_ms: u32,
    target_throughput: u32,
) -> Result<SimpleWasmBenchmarkResults, JsValue> {
    let config = create_simple_wasm_config();
    let mut client = SimpleWasmWebWorkersClient::new(websocket_url, config);
    
    let mut connection = client.connect().await?;
    let start_time = js_sys::Date::now();
    
    let test_frame = vec![0u8; 1024];
    let mut frames_sent = 0;
    
    while frames_sent < target_throughput {
        let elapsed = js_sys::Date::now() - start_time;
        if elapsed > duration_ms as f64 {
            break;
        }
        
        connection.send_frame(test_frame.clone()).await?;
        frames_sent += 1;
        
        if frames_sent % 100 == 0 {
            wasm_bindgen_futures::JsFuture::from(
                js_sys::Promise::resolve(&JsValue::from(1))
            ).await.ok();
        }
    }
    
    let elapsed = js_sys::Date::now() - start_time;
    Ok(SimpleWasmBenchmarkResults {
        messages_per_second: if elapsed > 0.0 { frames_sent as f64 / (elapsed / 1000.0) } else { 0.0 },
        total_messages: frames_sent,
        duration_ms: elapsed,
    })
}

#[derive(Debug)]
pub struct SimpleWasmBenchmarkResults {
    pub messages_per_second: f64,
    pub total_messages: u32,
    pub duration_ms: f64,
}
