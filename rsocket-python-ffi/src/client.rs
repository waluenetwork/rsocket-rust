use pyo3::prelude::*;
use rsocket_rust::prelude::*;
use crate::transport::PyTransportConfig;
use crate::payload::PyPayload;
use crate::get_runtime;

#[pyclass]
pub struct PyRSocketClient {
    client: Option<Box<dyn rsocket_rust::prelude::RSocket>>,
    transport_config: PyTransportConfig,
}

#[pymethods]
impl PyRSocketClient {
    #[new]
    pub fn new(transport_config: PyTransportConfig) -> PyResult<Self> {
        Ok(PyRSocketClient {
            client: None,
            transport_config,
        })
    }
    
    pub fn connect(&mut self) -> PyResult<()> {
        let runtime = get_runtime();
        let transport_config = self.transport_config.clone();
        
        let client = runtime.block_on(async move {
            let transport_config_clone = transport_config.clone();
            match transport_config_clone.transport_type.transport_type.as_str() {
                "tcp" => {
                    let transport = rsocket_rust_transport_tcp::TcpClientTransport::from(transport_config_clone.address.as_str());
                    let client = RSocketFactory::connect()
                        .transport(transport)
                        .start()
                        .await?;
                    Ok::<Box<dyn rsocket_rust::prelude::RSocket>, Box<dyn std::error::Error + Send + Sync>>(Box::new(client))
                },
                "websocket" => {
                    let transport = rsocket_rust_transport_websocket::WebsocketClientTransport::from(transport_config_clone.address.as_str());
                    let client = RSocketFactory::connect()
                        .transport(transport)
                        .start()
                        .await?;
                    Ok::<Box<dyn rsocket_rust::prelude::RSocket>, Box<dyn std::error::Error + Send + Sync>>(Box::new(client))
                },
                _ => Err(format!("Unsupported transport type: {}", transport_config_clone.transport_type.transport_type).into()),
            }
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Connection failed: {}", e)))?;
        
        self.client = Some(client);
        Ok(())
    }
    
    pub fn request_response(&self, payload: PyPayload) -> PyResult<Option<PyPayload>> {
        let client = self.client.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client not connected"))?;
        
        let runtime = get_runtime();
        let rsocket_payload = payload.to_rsocket_payload()?;
        let client_clone = client.clone();
        
        let result = runtime.block_on(async move {
            client_clone.request_response(rsocket_payload).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Request failed: {}", e)))?;
        
        match result {
            Some(payload) => Ok(Some(PyPayload::from_rsocket_payload(payload)?)),
            None => Ok(None),
        }
    }
    
    pub fn request_stream(&self, payload: PyPayload) -> PyResult<Vec<PyPayload>> {
        let client = self.client.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client not connected"))?;
        
        let runtime = get_runtime();
        let rsocket_payload = payload.to_rsocket_payload()?;
        let client_clone = client.clone();
        
        let results = runtime.block_on(async move {
            use futures::StreamExt;
            let mut stream = client_clone.request_stream(rsocket_payload);
            let mut results = Vec::new();
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(payload) => results.push(payload),
                    Err(e) => return Err(e),
                }
            }
            
            Ok(results)
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Stream request failed: {}", e)))?;
        
        results.into_iter()
            .map(|p| PyPayload::from_rsocket_payload(p))
            .collect()
    }
    
    pub fn fire_and_forget(&self, payload: PyPayload) -> PyResult<()> {
        let client = self.client.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client not connected"))?;
        
        let runtime = get_runtime();
        let rsocket_payload = payload.to_rsocket_payload()?;
        let client_clone = client.clone();
        
        runtime.block_on(async move {
            client_clone.fire_and_forget(rsocket_payload).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Fire and forget failed: {}", e)))?;
        
        Ok(())
    }
    
    pub fn request_channel(&self, payloads: Vec<PyPayload>) -> PyResult<Vec<PyPayload>> {
        let client = self.client.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Client not connected"))?;
        
        let runtime = get_runtime();
        let rsocket_payloads: Result<Vec<_>, _> = payloads.into_iter()
            .map(|p| p.to_rsocket_payload())
            .collect();
        let rsocket_payloads = rsocket_payloads?;
        let client_clone = client.clone();
        
        let results = runtime.block_on(async move {
            use futures::{StreamExt, stream};
            let input_stream = stream::iter(rsocket_payloads.into_iter().map(Ok));
            let mut output_stream = client_clone.request_channel(Box::pin(input_stream));
            let mut results = Vec::new();
            
            while let Some(result) = output_stream.next().await {
                match result {
                    Ok(payload) => results.push(payload),
                    Err(e) => return Err(e),
                }
            }
            
            Ok(results)
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Channel request failed: {}", e)))?;
        
        results.into_iter()
            .map(|p| PyPayload::from_rsocket_payload(p))
            .collect()
    }
    
    pub fn close(&mut self) -> PyResult<()> {
        self.client = None;
        Ok(())
    }
    
    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
    
    pub fn get_transport_type(&self) -> String {
        self.transport_config.transport_type.to_string()
    }
}
