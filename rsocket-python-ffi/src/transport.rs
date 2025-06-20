use pyo3::prelude::*;
use rsocket_rust_transport_tcp::{TcpClientTransport, TcpServerTransport};
use rsocket_rust_transport_websocket::{WebsocketClientTransport, WebsocketServerTransport};

#[pyclass(name = "TcpClientTransport")]
pub struct PyTcpClientTransport {
    address: String,
}

#[pymethods]
impl PyTcpClientTransport {
    #[new]
    pub fn new(address: String) -> Self {
        Self {
            address,
        }
    }
}

#[pyclass(name = "TcpServerTransport")]
pub struct PyTcpServerTransport {
    inner: TcpServerTransport,
}

#[pymethods]
impl PyTcpServerTransport {
    #[new]
    pub fn new(address: String) -> Self {
        Self {
            inner: TcpServerTransport::from(address),
        }
    }
}

#[pyclass(name = "WebSocketClientTransport")]
pub struct PyWebSocketClientTransport {
    url: String,
}

#[pymethods]
impl PyWebSocketClientTransport {
    #[new]
    pub fn new(url: String) -> Self {
        Self {
            url,
        }
    }
}

#[pyclass(name = "WebSocketServerTransport")]
pub struct PyWebSocketServerTransport {
    inner: WebsocketServerTransport,
}

#[pymethods]
impl PyWebSocketServerTransport {
    #[new]
    pub fn new(address: String) -> Self {
        Self {
            inner: WebsocketServerTransport::from(address.as_str()),
        }
    }
}

#[derive(Clone)]
#[pyclass(name = "QuinnClientTransport")]
pub struct PyQuinnClientTransport {
    address: String,
}

#[pymethods]
impl PyQuinnClientTransport {
    #[new]
    pub fn new(address: String) -> Self {
        Self {
            address,
        }
    }
}

#[derive(Clone)]
#[pyclass(name = "QuinnServerTransport")]
pub struct PyQuinnServerTransport {
    address: String,
}

#[pymethods]
impl PyQuinnServerTransport {
    #[new]
    pub fn new(address: String) -> Self {
        Self {
            address,
        }
    }
}

#[derive(Clone)]
#[pyclass(name = "IrohClientTransport")]
pub struct PyIrohClientTransport {
    address: String,
}

#[pymethods]
impl PyIrohClientTransport {
    #[new]
    pub fn new(address: String) -> Self {
        Self {
            address,
        }
    }
}

#[derive(Clone)]
#[pyclass(name = "IrohServerTransport")]
pub struct PyIrohServerTransport {
    address: String,
}

#[pymethods]
impl PyIrohServerTransport {
    #[new]
    pub fn new(address: String) -> Self {
        Self {
            address,
        }
    }
}

#[cfg(feature = "advanced-transports")]
mod advanced_transports {
    use super::*;
    use rsocket_rust_transport_quinn::webtransport::{WebTransportClientTransport, WebTransportServerTransport};
    use rsocket_rust_transport_wasm::webworkers::{WebWorkersClientTransport, WebWorkersConfig};
    
    #[pyclass(name = "WebTransportClientTransport")]
    pub struct PyWebTransportClientTransport {
        inner: WebTransportClientTransport,
    }
    
    #[pymethods]
    impl PyWebTransportClientTransport {
        #[new]
        pub fn new(url: String) -> Self {
            Self {
                inner: WebTransportClientTransport::new(url),
            }
        }
    }
    
    #[pyclass(name = "WebTransportServerTransport")]
    pub struct PyWebTransportServerTransport {
        inner: WebTransportServerTransport,
    }
    
    #[pymethods]
    impl PyWebTransportServerTransport {
        #[new]
        pub fn new(address: String) -> Self {
            Self {
                inner: WebTransportServerTransport::new(address),
            }
        }
    }
    
    #[pyclass(name = "WebWorkersClientTransport")]
    pub struct PyWebWorkersClientTransport {
        inner: WebWorkersClientTransport,
    }
    
    #[pymethods]
    impl PyWebWorkersClientTransport {
        #[new]
        pub fn new(url: String, config: &PyWebWorkersConfig) -> Self {
            Self {
                inner: WebWorkersClientTransport::new(url, config.inner.clone()),
            }
        }
    }
    
    #[pyclass(name = "WebWorkersConfig")]
    pub struct PyWebWorkersConfig {
        pub(crate) inner: WebWorkersConfig,
    }
    
    #[pymethods]
    impl PyWebWorkersConfig {
        #[new]
        pub fn new() -> Self {
            Self {
                inner: WebWorkersConfig::default(),
            }
        }
        
        pub fn with_worker_count(mut self, count: usize) -> Self {
            self.inner.worker_count = count;
            self
        }
        
        pub fn with_buffer_size(mut self, size: usize) -> Self {
            self.inner.buffer_size = size;
            self
        }
    }
}

#[cfg(feature = "advanced-transports")]
pub use advanced_transports::*;

impl From<PyTcpClientTransport> for TcpClientTransport {
    fn from(transport: PyTcpClientTransport) -> Self {
        TcpClientTransport::from(transport.address)
    }
}

impl From<&PyTcpClientTransport> for TcpClientTransport {
    fn from(transport: &PyTcpClientTransport) -> Self {
        TcpClientTransport::from(transport.address.clone())
    }
}

impl From<PyWebSocketClientTransport> for WebsocketClientTransport {
    fn from(transport: PyWebSocketClientTransport) -> Self {
        WebsocketClientTransport::from(transport.url.as_str())
    }
}

impl From<&PyWebSocketClientTransport> for WebsocketClientTransport {
    fn from(transport: &PyWebSocketClientTransport) -> Self {
        WebsocketClientTransport::from(transport.url.as_str())
    }
}

// impl From<PyQuinnClientTransport> for QuinnClientTransport {
//     fn from(transport: PyQuinnClientTransport) -> Self {
//     }
// }

// impl From<PyIrohClientTransport> for IrohClientTransport {
//     fn from(transport: PyIrohClientTransport) -> Self {
//     }
// }
