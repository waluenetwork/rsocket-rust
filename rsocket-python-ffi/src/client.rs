use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use rsocket_rust::prelude::*;
use crate::payload::PyPayload;

use std::sync::Arc;

#[pyclass(name = "Client")]
pub struct PyClient {
    inner: Arc<dyn RSocket>,
}

#[pymethods]
impl PyClient {
    pub fn request_response<'p>(&self, py: Python<'p>, payload: PyPayload) -> PyResult<&'p PyAny> {
        let client = self.inner.clone();
        let payload: Payload = payload.into();
        
        future_into_py(py, async move {
            match client.request_response(payload).await {
                Ok(Some(response)) => Ok(PyPayload::from(response)),
                Ok(None) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("No response received")),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Request failed: {}", e))),
            }
        })
    }
    
    pub fn fire_and_forget<'p>(&self, py: Python<'p>, payload: PyPayload) -> PyResult<&'p PyAny> {
        let client = self.inner.clone();
        let payload: Payload = payload.into();
        
        future_into_py(py, async move {
            match client.fire_and_forget(payload).await {
                Ok(_) => Ok(()),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Fire and forget failed: {}", e))),
            }
        })
    }
}

impl PyClient {
    pub fn new(client: Box<dyn RSocket>) -> Self {
        PyClient { inner: Arc::from(client) }
    }
}
