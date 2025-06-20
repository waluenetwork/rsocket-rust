use pyo3::prelude::*;

use crate::transport::{PyTransportConfig, PyTransportType};
use crate::client::PyRSocketClient;
use crate::server::PyRSocketServer;

#[pyclass]
pub struct PyRSocketFactory;

#[pymethods]
impl PyRSocketFactory {
    #[new]
    pub fn new() -> Self {
        PyRSocketFactory
    }
    
    #[staticmethod]
    pub fn create_tcp_client(address: String) -> PyResult<PyRSocketClient> {
        let config = PyTransportConfig::new(
            PyTransportType::tcp(),
            address,
            None,
        );
        PyRSocketClient::new(config)
    }
    
    #[staticmethod]
    pub fn create_websocket_client(address: String) -> PyResult<PyRSocketClient> {
        let config = PyTransportConfig::new(
            PyTransportType::websocket(),
            address,
            None,
        );
        PyRSocketClient::new(config)
    }
    
    #[staticmethod]
    pub fn create_wasm_client(_address: String, _options: Option<String>) -> PyResult<PyRSocketClient> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>("WASM transport not yet implemented"))
    }
    
    #[staticmethod]
    pub fn create_optimized_client(transport_type: String, address: String, enable_simd: Option<bool>) -> PyResult<PyRSocketClient> {
        let transport_type = match transport_type.as_str() {
            "tcp" => PyTransportType::tcp(),
            "websocket" => PyTransportType::websocket(),

            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unsupported transport type: {}", transport_type))),
        };
        
        let mut config = PyTransportConfig::new(transport_type, address, None);
        
        if enable_simd.unwrap_or(true) {
            config.enable_simd_processing();
        }
        
        config.set_performance_mode("high".to_string());
        
        PyRSocketClient::new(config)
    }
    
    #[staticmethod]
    pub fn create_tcp_server(address: String) -> PyResult<PyRSocketServer> {
        let config = PyTransportConfig::new(
            PyTransportType::tcp(),
            address,
            None,
        );
        PyRSocketServer::new(config)
    }
    
    #[staticmethod]
    pub fn create_websocket_server(address: String) -> PyResult<PyRSocketServer> {
        let config = PyTransportConfig::new(
            PyTransportType::websocket(),
            address,
            None,
        );
        PyRSocketServer::new(config)
    }
    
    #[staticmethod]
    pub fn create_wasm_server(_address: String) -> PyResult<PyRSocketServer> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>("WASM transport not yet implemented"))
    }
}
