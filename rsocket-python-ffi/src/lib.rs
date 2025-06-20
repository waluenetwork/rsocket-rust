use pyo3::prelude::*;
use pyo3::types::PyBytes;
use tokio::runtime::Runtime;

mod client;
mod server;
mod payload;
mod transport;
mod factory;
mod performance;

use client::PyRSocketClient;
use server::PyRSocketServer;
use payload::PyPayload;
use transport::{PyTransportConfig, PyTransportType};
use factory::PyRSocketFactory;
use performance::PyPerformanceMetrics;

#[pymodule]
fn rsocket_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    env_logger::init();
    
    m.add_class::<PyRSocketClient>()?;
    m.add_class::<PyRSocketServer>()?;
    m.add_class::<PyPayload>()?;
    m.add_class::<PyTransportConfig>()?;
    m.add_class::<PyTransportType>()?;
    m.add_class::<PyRSocketFactory>()?;
    m.add_class::<PyPerformanceMetrics>()?;
    
    m.add_function(wrap_pyfunction!(create_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_server, m)?)?;
    m.add_function(wrap_pyfunction!(create_payload, m)?)?;
    m.add_function(wrap_pyfunction!(get_supported_transports, m)?)?;
    m.add_function(wrap_pyfunction!(get_performance_metrics, m)?)?;
    m.add_function(wrap_pyfunction!(enable_crossbeam_optimizations, m)?)?;
    
    Ok(())
}

#[pyfunction]
fn create_client(transport_config: PyTransportConfig) -> PyResult<PyRSocketClient> {
    PyRSocketClient::new(transport_config)
}

#[pyfunction]
fn create_server(transport_config: PyTransportConfig) -> PyResult<PyRSocketServer> {
    PyRSocketServer::new(transport_config)
}

#[pyfunction]
fn create_payload(data: Option<&PyBytes>, metadata: Option<&PyBytes>) -> PyResult<PyPayload> {
    PyPayload::new(data, metadata)
}

#[pyfunction]
fn get_supported_transports() -> PyResult<Vec<String>> {
    Ok(vec![
        "tcp".to_string(),
        "websocket".to_string(),
        "quinn-quic".to_string(),
        "quinn-webtransport".to_string(),
        "iroh-roq".to_string(),
        "wasm-webworkers".to_string(),
        "iroh-p2p".to_string(),
        "iroh-p2p-wasm".to_string(),
    ])
}

#[pyfunction]
fn get_performance_metrics() -> PyResult<PyPerformanceMetrics> {
    PyPerformanceMetrics::new()
}

#[pyfunction]
fn enable_crossbeam_optimizations(enabled: bool) -> PyResult<bool> {
    Ok(enabled)
}

pub(crate) fn get_runtime() -> &'static Runtime {
    static RUNTIME: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create Tokio runtime")
    })
}
