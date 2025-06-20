use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

#[napi(object)]
#[derive(Clone)]
pub struct JsTransportType {
    pub transport_type: String,
}

#[napi]
impl JsTransportType {
    #[napi(constructor)]
    pub fn new(transport_type: String) -> Self {
        JsTransportType { transport_type }
    }
    
    #[napi]
    pub fn to_string(&self) -> String {
        self.transport_type.clone()
    }
    
    #[napi(factory)]
    pub fn tcp() -> Self {
        JsTransportType::new("tcp".to_string())
    }
    
    #[napi(factory)]
    pub fn websocket() -> Self {
        JsTransportType::new("websocket".to_string())
    }
    
    #[napi(factory)]
    pub fn quinn_quic() -> Self {
        JsTransportType::new("quinn-quic".to_string())
    }
    
    #[napi(factory)]
    pub fn quinn_webtransport() -> Self {
        JsTransportType::new("quinn-webtransport".to_string())
    }
    
    #[napi(factory)]
    pub fn iroh_roq() -> Self {
        JsTransportType::new("iroh-roq".to_string())
    }
    
    #[napi(factory)]
    pub fn wasm_webworkers() -> Self {
        JsTransportType::new("wasm-webworkers".to_string())
    }
    
    #[napi(factory)]
    pub fn iroh_p2p() -> Self {
        JsTransportType::new("iroh-p2p".to_string())
    }
    
    #[napi(factory)]
    pub fn iroh_p2p_wasm() -> Self {
        JsTransportType::new("iroh-p2p-wasm".to_string())
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct JsTransportConfig {
    pub transport_type: JsTransportType,
    pub address: String,
    pub options: HashMap<String, String>,
}

#[napi]
impl JsTransportConfig {
    #[napi(constructor)]
    pub fn new(transport_type: JsTransportType, address: String, options: Option<HashMap<String, String>>) -> Self {
        JsTransportConfig {
            transport_type,
            address,
            options: options.unwrap_or_default(),
        }
    }
    
    #[napi]
    pub fn set_option(&mut self, key: String, value: String) {
        self.options.insert(key, value);
    }
    
    #[napi]
    pub fn get_option(&self, key: String) -> Option<String> {
        self.options.get(&key).cloned()
    }
    
    #[napi]
    pub fn enable_crossbeam_optimizations(&mut self) {
        self.options.insert("crossbeam_optimizations".to_string(), "true".to_string());
    }
    
    #[napi]
    pub fn enable_simd_processing(&mut self) {
        self.options.insert("simd_processing".to_string(), "true".to_string());
    }
    
    #[napi]
    pub fn enable_webworkers(&mut self) {
        self.options.insert("webworkers_enabled".to_string(), "true".to_string());
    }
    
    #[napi]
    pub fn set_webworkers_count(&mut self, count: u32) {
        self.options.insert("webworkers_count".to_string(), count.to_string());
    }
    
    #[napi]
    pub fn set_performance_mode(&mut self, mode: String) {
        self.options.insert("performance_mode".to_string(), mode);
    }
    
    #[napi]
    pub fn set_buffer_size(&mut self, size: u32) {
        self.options.insert("buffer_size".to_string(), size.to_string());
    }
    
    #[napi]
    pub fn enable_compression(&mut self, enabled: bool) {
        self.options.insert("compression_enabled".to_string(), enabled.to_string());
    }
    
    #[napi]
    pub fn to_string(&self) -> String {
        format!("JsTransportConfig({}, {})", self.transport_type.transport_type, self.address)
    }
    
    #[napi]
    pub fn get_all_options(&self) -> HashMap<String, String> {
        self.options.clone()
    }
}
