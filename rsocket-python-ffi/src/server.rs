use pyo3::prelude::*;
use rsocket_rust::prelude::*;
use crate::transport::PyTransportConfig;
use crate::payload::PyPayload;
use crate::get_runtime;
use async_trait::async_trait;

#[pyclass]
pub struct PyRSocketServer {
    server: Option<tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
    transport_config: PyTransportConfig,
    request_handler: Option<PyObject>,
}

#[pymethods]
impl PyRSocketServer {
    #[new]
    pub fn new(transport_config: PyTransportConfig) -> PyResult<Self> {
        Ok(PyRSocketServer {
            server: None,
            transport_config,
            request_handler: None,
        })
    }
    
    pub fn set_request_handler(&mut self, handler: PyObject) {
        self.request_handler = Some(handler);
    }
    
    pub fn start(&mut self) -> PyResult<()> {
        let runtime = get_runtime();
        let transport_config = self.transport_config.clone();
        let handler = self.request_handler.clone();
        
        let server_handle = runtime.spawn(async move {
            match transport_config.transport_type.transport_type.as_str() {
                "tcp" => {
                    let mut transport = rsocket_rust_transport_tcp::TcpServerTransport::from(transport_config.address.as_str());
                    transport.start().await?;
                    println!("TCP RSocket server started on {}", transport_config.address);
                    
                    while let Some(conn_result) = transport.next().await {
                        match conn_result {
                            Ok(_conn) => {
                                println!("New TCP connection accepted");
                            },
                            Err(e) => {
                                eprintln!("TCP connection error: {}", e);
                            }
                        }
                    }
                },
                "websocket" => {
                    let mut transport = rsocket_rust_transport_websocket::WebsocketServerTransport::from(transport_config.address.as_str());
                    transport.start().await?;
                    println!("WebSocket RSocket server started on {}", transport_config.address);
                    
                    while let Some(conn_result) = transport.next().await {
                        match conn_result {
                            Ok(_conn) => {
                                println!("New WebSocket connection accepted");
                            },
                            Err(e) => {
                                eprintln!("WebSocket connection error: {}", e);
                            }
                        }
                    }
                },
                _ => return Err(format!("Unsupported server transport type: {}", transport_config.transport_type.transport_type).into()),
            }
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });
        
        self.server = Some(server_handle);
        Ok(())
    }
    
    pub fn stop(&mut self) -> PyResult<()> {
        if let Some(server) = self.server.take() {
            server.abort();
        }
        Ok(())
    }
    
    pub fn is_running(&self) -> bool {
        self.server.is_some()
    }
    
    pub fn get_transport_type(&self) -> String {
        self.transport_config.transport_type.to_string()
    }
}

struct PyRequestHandler {
    handler: Option<PyObject>,
}

impl PyRequestHandler {
    fn new(handler: Option<PyObject>) -> Self {
        PyRequestHandler { handler }
    }
}

#[async_trait]
impl RSocket for PyRequestHandler {
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>, anyhow::Error> {
        if let Some(handler) = &self.handler {
            Python::with_gil(|py| {
                let py_payload = PyPayload::from_rsocket_payload(req)
                    .map_err(|e| anyhow::Error::new(e))?;
                
                let result = handler.call_method1(py, "request_response", (py_payload,))
                    .map_err(|e| anyhow::Error::new(e))?;
                
                if result.is_none(py) {
                    Ok(None)
                } else {
                    let py_result: PyPayload = result.extract(py)
                        .map_err(|e| anyhow::Error::new(e))?;
                    let rsocket_payload = py_result.to_rsocket_payload()
                        .map_err(|e| anyhow::Error::new(e))?;
                    Ok(Some(rsocket_payload))
                }
            })
        } else {
            Ok(None)
        }
    }
    
    async fn fire_and_forget(&self, req: Payload) -> Result<(), anyhow::Error> {
        if let Some(handler) = &self.handler {
            Python::with_gil(|py| {
                let py_payload = PyPayload::from_rsocket_payload(req)
                    .map_err(|e| anyhow::Error::new(e))?;
                
                handler.call_method1(py, "fire_and_forget", (py_payload,))
                    .map_err(|e| anyhow::Error::new(e))?;
                
                Ok(())
            })
        } else {
            Ok(())
        }
    }
    
    async fn metadata_push(&self, _req: Payload) -> Result<(), anyhow::Error> {
        Ok(())
    }
    
    fn request_stream(&self, _req: Payload) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Payload, anyhow::Error>> + Send + 'static>> {
        use futures::stream;
        Box::pin(stream::empty())
    }
    
    fn request_channel(&self, _reqs: std::pin::Pin<Box<dyn futures::Stream<Item = Result<Payload, anyhow::Error>> + Send + 'static>>) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Payload, anyhow::Error>> + Send + 'static>> {
        use futures::stream;
        Box::pin(stream::empty())
    }
}
