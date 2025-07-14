use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use rsocket_rust::Client;
use rsocket_rust::prelude::RSocket;
use crate::payload::PyPayload;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[pyclass(name = "Client")]
pub struct PyClient {
    inner: Client,
}

#[pymethods]
impl PyClient {
    fn metadata_push<'py>(&self, py: Python<'py>, payload: PyPayload) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            client.metadata_push(rust_payload).await
                .map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("MetadataPush failed"))
        })
    }

    fn fire_and_forget<'py>(&self, py: Python<'py>, payload: PyPayload) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            client.fire_and_forget(rust_payload).await
                .map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("FireAndForget failed"))
        })
    }

    fn request_response<'py>(&self, py: Python<'py>, payload: PyPayload) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            match client.request_response(rust_payload).await {
                Ok(Some(response)) => Ok(Some(PyPayload::from_rust(response))),
                Ok(None) => Ok(None),
                Err(_) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("RequestResponse failed")),
            }
        })
    }

    fn request_stream<'py>(&self, py: Python<'py>, payload: PyPayload) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            let stream = client.request_stream(rust_payload);
            Ok(PyStreamIterator::new(stream))
        })
    }

    fn request_channel<'py>(&self, py: Python<'py>, payloads: Vec<PyPayload>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payloads: Vec<_> = payloads.into_iter().map(|p| p.to_rust()).collect();
        
        future_into_py(py, async move {
            let stream = futures::stream::iter(rust_payloads.into_iter().map(Ok));
            let response_stream = client.request_channel(Box::pin(stream));
            Ok(PyStreamIterator::new(response_stream))
        })
    }

    fn __str__(&self) -> String {
        "RSocket Client".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyClient {
    pub fn from_rust(client: Client) -> Self {
        PyClient { inner: client }
    }
}

#[pyclass(name = "StreamIterator")]
pub struct PyStreamIterator {
    inner: Arc<Mutex<Option<rsocket_rust::spi::Flux<rsocket_rust::Result<rsocket_rust::payload::Payload>>>>>,
}

impl PyStreamIterator {
    pub fn new(stream: rsocket_rust::spi::Flux<rsocket_rust::Result<rsocket_rust::payload::Payload>>) -> Self {
        PyStreamIterator {
            inner: Arc::new(Mutex::new(Some(stream))),
        }
    }
}

#[pymethods]
impl PyStreamIterator {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        
        future_into_py(py, async move {
            let mut guard = inner.lock().await;
            if let Some(ref mut stream) = guard.as_mut() {
                match stream.next().await {
                    Some(Ok(payload)) => Ok(Some(PyPayload::from_rust(payload))),
                    Some(Err(_)) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Stream error")),
                    None => {
                        *guard = None;
                        Err(PyErr::new::<pyo3::exceptions::PyStopAsyncIteration, _>(""))
                    }
                }
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyStopAsyncIteration, _>(""))
            }
        })
    }

    fn __str__(&self) -> String {
        "StreamIterator".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}
