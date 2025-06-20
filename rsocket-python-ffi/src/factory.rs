use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use crate::client::PyClient;
use crate::transport::*;

#[pyclass(name = "RSocketFactory")]
pub struct PyRSocketFactory;

#[pymethods]
impl PyRSocketFactory {
    #[staticmethod]
    pub fn connect_tcp<'p>(py: Python<'p>, transport: &PyTcpClientTransport) -> PyResult<&'p PyAny> {
        let transport: TcpClientTransport = transport.into();
        
        future_into_py(py, async move {
            match RSocketFactory::connect()
                .transport(transport)
                .start()
                .await
            {
                Ok(client) => Ok(PyClient::new(Box::new(client))),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!("Connection failed: {}", e))),
            }
        })
    }
    
    #[staticmethod]
    pub fn connect_websocket<'p>(py: Python<'p>, transport: &PyWebSocketClientTransport) -> PyResult<&'p PyAny> {
        let transport: WebsocketClientTransport = transport.into();
        
        future_into_py(py, async move {
            match RSocketFactory::connect()
                .transport(transport)
                .start()
                .await
            {
                Ok(client) => Ok(PyClient::new(Box::new(client))),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!("Connection failed: {}", e))),
            }
        })
    }
    
    #[staticmethod]
    pub fn connect_quic<'p>(_py: Python<'p>, _transport: &PyQuinnClientTransport) -> PyResult<&'p PyAny> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>("Quinn transport not yet implemented"))
    }
    
    #[staticmethod]
    pub fn connect_iroh<'p>(_py: Python<'p>, _transport: &PyIrohClientTransport) -> PyResult<&'p PyAny> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>("Iroh transport not yet implemented"))
    }
}
