use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use rsocket_rust::Client;
use rsocket_rust::prelude::RSocket;
use crate::payload::PyPayload;
use futures::StreamExt;

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
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("MetadataPush failed: {}", e)))
        })
    }

    fn fire_and_forget<'py>(&self, py: Python<'py>, payload: PyPayload) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            client.fire_and_forget(rust_payload).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("FireAndForget failed: {}", e)))
        })
    }

    fn request_response<'py>(&self, py: Python<'py>, payload: PyPayload) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            match client.request_response(rust_payload).await {
                Ok(Some(response)) => Ok(Some(PyPayload::from_rust(response))),
                Ok(None) => Ok(None),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("RequestResponse failed: {}", e))),
            }
        })
    }

    fn request_stream<'py>(&self, py: Python<'py>, payload: PyPayload) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            let mut stream = client.request_stream(rust_payload);
            let mut results = Vec::new();
            while let Some(item) = stream.next().await {
                match item {
                    Ok(payload) => results.push(PyPayload::from_rust(payload)),
                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Stream error: {}", e))),
                }
            }
            Ok(results)
        })
    }

    fn request_stream_with_callback<'py>(&self, py: Python<'py>, payload: PyPayload, on_next: PyObject, on_complete: Option<PyObject>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payload = payload.to_rust();
        
        future_into_py(py, async move {
            let mut stream = client.request_stream(rust_payload);
            let mut item_count = 0;
            let mut stream_error: Option<String> = None;
            
            while let Some(item) = stream.next().await {
                match item {
                    Ok(payload) => {
                        item_count += 1;
                        let py_payload = PyPayload::from_rust(payload);
                        
                        let callback_result = Python::with_gil(|py| {
                            on_next.call1(py, (py_payload, item_count))
                        });
                        
                        if let Err(e) = callback_result {
                            stream_error = Some(format!("Callback error: {}", e));
                            break;
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    },
                    Err(e) => {
                        stream_error = Some(format!("Stream error: {}", e));
                        break;
                    }
                }
            }
            
            if let Some(on_complete_callback) = on_complete {
                let completion_result = Python::with_gil(|py| {
                    if let Some(ref error) = stream_error {
                        on_complete_callback.call1(py, (item_count, py.None(), error.clone()))
                    } else {
                        on_complete_callback.call1(py, (item_count, true, py.None()))
                    }
                });
                
                if let Err(e) = completion_result {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Completion callback error: {}", e)));
                }
            }
            
            if let Some(error) = stream_error {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error));
            }
            
            Ok(item_count)
        })
    }

    fn request_channel<'py>(&self, py: Python<'py>, payloads: Vec<PyPayload>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payloads: Vec<_> = payloads.into_iter().map(|p| p.to_rust()).collect();
        
        future_into_py(py, async move {
            let stream = futures::stream::iter(rust_payloads.into_iter().map(Ok));
            let mut response_stream = client.request_channel(Box::pin(stream));
            let mut results = Vec::new();
            while let Some(item) = response_stream.next().await {
                match item {
                    Ok(payload) => results.push(PyPayload::from_rust(payload)),
                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Channel error: {}", e))),
                }
            }
            Ok(results)
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
