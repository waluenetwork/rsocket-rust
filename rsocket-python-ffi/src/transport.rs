use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use rsocket_rust_transport_tcp::{TcpClientTransport, TcpServerTransport};
use rsocket_rust_transport_websocket::{WebsocketClientTransport, WebsocketServerTransport};
use rsocket_rust_transport_quinn::{QuinnClientTransport, QuinnServerTransport};
use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};
use std::net::SocketAddr;

#[pyclass(name = "TcpClientTransport")]
#[derive(Clone)]
pub struct PyTcpClientTransport {
    addr: String,
}

#[pymethods]
impl PyTcpClientTransport {
    #[new]
    fn new(addr: &str) -> PyResult<Self> {
        Ok(PyTcpClientTransport {
            addr: addr.to_string(),
        })
    }

    fn __str__(&self) -> String {
        "TcpClientTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyTcpClientTransport {
    pub fn to_rust(self) -> TcpClientTransport {
        TcpClientTransport::from(self.addr.as_str())
    }
}

#[pyclass(name = "TcpServerTransport")]
#[derive(Clone)]
pub struct PyTcpServerTransport {
    addr: String,
}

#[pymethods]
impl PyTcpServerTransport {
    #[new]
    fn new(addr: &str) -> PyResult<Self> {
        let _socket_addr: SocketAddr = addr.parse()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid address: {}", e)))?;
        Ok(PyTcpServerTransport {
            addr: addr.to_string(),
        })
    }

    fn __str__(&self) -> String {
        "TcpServerTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyTcpServerTransport {
    pub fn to_rust(self) -> TcpServerTransport {
        let socket_addr: SocketAddr = self.addr.parse().expect("Invalid address");
        TcpServerTransport::from(socket_addr)
    }
}

#[pyclass(name = "WebSocketClientTransport")]
#[derive(Clone)]
pub struct PyWebSocketClientTransport {
    url: String,
}

#[pymethods]
impl PyWebSocketClientTransport {
    #[new]
    fn new(url: &str) -> PyResult<Self> {
        Ok(PyWebSocketClientTransport {
            url: url.to_string(),
        })
    }

    fn __str__(&self) -> String {
        "WebSocketClientTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyWebSocketClientTransport {
    pub fn to_rust(self) -> WebsocketClientTransport {
        WebsocketClientTransport::from(self.url.as_str())
    }
}

#[pyclass(name = "WebSocketServerTransport")]
#[derive(Clone)]
pub struct PyWebSocketServerTransport {
    addr: String,
}

#[pymethods]
impl PyWebSocketServerTransport {
    #[new]
    fn new(addr: &str) -> PyResult<Self> {
        let _socket_addr: SocketAddr = addr.parse()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid address: {}", e)))?;
        Ok(PyWebSocketServerTransport {
            addr: addr.to_string(),
        })
    }

    fn __str__(&self) -> String {
        "WebSocketServerTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyWebSocketServerTransport {
    pub fn to_rust(self) -> WebsocketServerTransport {
        let socket_addr: SocketAddr = self.addr.parse().expect("Invalid address");
        WebsocketServerTransport::from(socket_addr)
    }
}

#[pyclass(name = "QuinnClientTransport")]
#[derive(Clone)]
pub struct PyQuinnClientTransport {
    addr: String,
}

#[pymethods]
impl PyQuinnClientTransport {
    #[new]
    fn new(addr: &str) -> PyResult<Self> {
        Ok(PyQuinnClientTransport {
            addr: addr.to_string(),
        })
    }

    fn __str__(&self) -> String {
        "QuinnClientTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyQuinnClientTransport {
    pub fn to_rust(self) -> QuinnClientTransport {
        QuinnClientTransport::from(self.addr.as_str())
    }
}

#[pyclass(name = "QuinnServerTransport")]
#[derive(Clone)]
pub struct PyQuinnServerTransport {
    addr: String,
}

#[pymethods]
impl PyQuinnServerTransport {
    #[new]
    fn new(addr: &str) -> PyResult<Self> {
        let _socket_addr: SocketAddr = addr.parse()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid address: {}", e)))?;
        Ok(PyQuinnServerTransport {
            addr: addr.to_string(),
        })
    }

    fn __str__(&self) -> String {
        "QuinnServerTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyQuinnServerTransport {
    pub fn to_rust(self) -> QuinnServerTransport {
        let socket_addr: SocketAddr = self.addr.parse().expect("Invalid address");
        QuinnServerTransport::from(socket_addr)
    }
}

#[pyclass(name = "IrohClientTransport")]
#[derive(Clone)]
pub struct PyIrohClientTransport {
    addr: String,
}

#[pymethods]
impl PyIrohClientTransport {
    #[new]
    fn new(addr: &str) -> PyResult<Self> {
        Ok(PyIrohClientTransport {
            addr: addr.to_string(),
        })
    }

    fn __str__(&self) -> String {
        "IrohClientTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyIrohClientTransport {
    pub fn to_rust(self) -> IrohClientTransport {
        IrohClientTransport::from(self.addr.as_str())
    }
}

#[pyclass(name = "IrohServerTransport")]
#[derive(Clone)]
pub struct PyIrohServerTransport {
    config: Option<rsocket_rust_transport_iroh::misc::IrohConfig>,
    node_id: Option<String>,
}

#[pymethods]
impl PyIrohServerTransport {
    #[new]
    fn new(private_key: Option<String>) -> PyResult<Self> {
        let mut config = rsocket_rust_transport_iroh::misc::IrohConfig::default();
        if let Some(key) = private_key {
            config.private_key = Some(key);
        }
        Ok(PyIrohServerTransport {
            config: Some(config),
            node_id: None,
        })
    }

    fn node_id(&self) -> Option<String> {
        self.node_id.clone()
    }

    fn get_node_id_after_start<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let config = self.config.clone().unwrap_or_default();
        future_into_py(py, async move {
            let mut server_transport = IrohServerTransport::from(config);
            server_transport.start().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start transport: {}", e)))?;
            
            Ok(server_transport.node_id())
        })
    }

    fn __str__(&self) -> String {
        "IrohServerTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyIrohServerTransport {
    pub fn to_rust(mut self) -> IrohServerTransport {
        let config = self.config.take().unwrap_or_default();
        let transport = IrohServerTransport::from(config);
        
        if let Some(node_id) = transport.node_id() {
            self.node_id = Some(node_id);
        }
        
        transport
    }
}
