use pyo3::prelude::*;
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
}

#[pymethods]
impl PyIrohServerTransport {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(PyIrohServerTransport {})
    }

    fn __str__(&self) -> String {
        "IrohServerTransport".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyIrohServerTransport {
    pub fn to_rust(self) -> IrohServerTransport {
        IrohServerTransport::default()
    }
}
