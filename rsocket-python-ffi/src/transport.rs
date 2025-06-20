use pyo3::prelude::*;
use std::collections::HashMap;

use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;

#[pyclass]
#[derive(Clone)]
pub struct PyTransportType {
    pub transport_type: String,
}

#[pymethods]
impl PyTransportType {
    #[new]
    pub fn new(transport_type: String) -> Self {
        PyTransportType { transport_type }
    }
    
    pub fn __str__(&self) -> String {
        self.transport_type.clone()
    }
    
    pub fn __repr__(&self) -> String {
        format!("PyTransportType('{}')", self.transport_type)
    }
    
    #[staticmethod]
    pub fn tcp() -> Self {
        PyTransportType::new("tcp".to_string())
    }
    
    #[staticmethod]
    pub fn websocket() -> Self {
        PyTransportType::new("websocket".to_string())
    }
    

}

impl ToString for PyTransportType {
    fn to_string(&self) -> String {
        self.transport_type.clone()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyTransportConfig {
    pub transport_type: PyTransportType,
    pub address: String,
    pub options: HashMap<String, String>,
}

#[pymethods]
impl PyTransportConfig {
    #[new]
    pub fn new(transport_type: PyTransportType, address: String, options: Option<HashMap<String, String>>) -> Self {
        PyTransportConfig {
            transport_type,
            address,
            options: options.unwrap_or_default(),
        }
    }
    
    pub fn set_option(&mut self, key: String, value: String) {
        self.options.insert(key, value);
    }
    
    pub fn get_option(&self, key: String) -> Option<String> {
        self.options.get(&key).cloned()
    }
    
    pub fn enable_crossbeam_optimizations(&mut self) {
        self.options.insert("crossbeam_optimizations".to_string(), "true".to_string());
    }
    
    pub fn enable_simd_processing(&mut self) {
        self.options.insert("simd_processing".to_string(), "true".to_string());
    }
    

    
    pub fn set_performance_mode(&mut self, mode: String) {
        self.options.insert("performance_mode".to_string(), mode);
    }
    
    pub fn __str__(&self) -> String {
        format!("PyTransportConfig({}, {})", self.transport_type.transport_type, self.address)
    }
    
    pub fn __repr__(&self) -> String {
        format!("PyTransportConfig(transport_type='{}', address='{}', options={:?})", 
                self.transport_type.transport_type, self.address, self.options)
    }
}

pub fn create_transport(config: PyTransportConfig) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error + Send + Sync>> {
    match config.transport_type.transport_type.as_str() {
        "tcp" => {
            let transport = TcpClientTransport::from(config.address.as_str());
            Ok(Box::new(transport) as Box<dyn std::any::Any>)
        },
        "websocket" => {
            let transport = WebsocketClientTransport::from(config.address.as_str());
            Ok(Box::new(transport) as Box<dyn std::any::Any>)
        },
        _ => Err(format!("Unsupported transport type: {}", config.transport_type.transport_type).into()),
    }
}
