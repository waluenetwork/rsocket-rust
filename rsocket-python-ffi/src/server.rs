use pyo3::prelude::*;
use crate::transport::*;

#[pyclass(name = "MultiTransportServerBuilder")]
pub struct PyMultiTransportServerBuilder {
    transports: Vec<String>,
}

#[pymethods]
impl PyMultiTransportServerBuilder {
    #[new]
    pub fn new() -> Self {
        Self {
            transports: Vec::new(),
        }
    }
    
    pub fn add_tcp_transport(&mut self, name: String, _transport: &PyTcpServerTransport) {
        self.transports.push(format!("TCP: {} - {}", name, "configured"));
    }
    
    pub fn add_websocket_transport(&mut self, name: String, _transport: &PyWebSocketServerTransport) {
        self.transports.push(format!("WebSocket: {} - {}", name, "configured"));
    }
    
    pub fn add_quic_transport(&mut self, name: String, _transport: &PyQuinnServerTransport) {
        self.transports.push(format!("QUIC: {} - {}", name, "configured"));
    }
    
    pub fn add_iroh_transport(&mut self, name: String, _transport: &PyIrohServerTransport) {
        self.transports.push(format!("Iroh: {} - {}", name, "configured"));
    }
    
    pub fn get_configured_transports(&self) -> Vec<String> {
        self.transports.clone()
    }
}
