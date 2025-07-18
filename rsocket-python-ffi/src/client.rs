use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use rsocket_rust::Client;
use rsocket_rust::prelude::RSocket;
use crate::payload::PyPayload;
use futures::StreamExt;
use async_stream::stream;
use anyhow;

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

    fn request_channel_with_callback<'py>(&self, py: Python<'py>, input_payloads: Vec<PyPayload>, on_response: PyObject, on_complete: Option<PyObject>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let rust_payloads: Vec<_> = input_payloads.into_iter().map(|p| p.to_rust()).collect();
        
        future_into_py(py, async move {
            let input_stream = futures::stream::iter(rust_payloads.into_iter().map(Ok));
            let mut response_stream = client.request_channel(Box::pin(input_stream));
            let mut response_count = 0;
            let mut channel_error: Option<String> = None;
            
            while let Some(item) = response_stream.next().await {
                match item {
                    Ok(payload) => {
                        response_count += 1;
                        let py_payload = PyPayload::from_rust(payload);
                        
                        let callback_result = Python::with_gil(|py| {
                            on_response.call1(py, (py_payload, response_count))
                        });
                        
                        if let Err(e) = callback_result {
                            channel_error = Some(format!("Response callback error: {}", e));
                            break;
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    },
                    Err(e) => {
                        channel_error = Some(format!("Channel error: {}", e));
                        break;
                    }
                }
            }
            
            if let Some(on_complete_callback) = on_complete {
                let completion_result = Python::with_gil(|py| {
                    if let Some(ref error) = channel_error {
                        on_complete_callback.call1(py, (response_count, py.None(), error.clone()))
                    } else {
                        on_complete_callback.call1(py, (response_count, true, py.None()))
                    }
                });
                
                if let Err(e) = completion_result {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Completion callback error: {}", e)));
                }
            }
            
            if let Some(error) = channel_error {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error));
            }
            
            Ok(response_count)
        })
    }

    fn request_channel_reactive<'py>(&self, py: Python<'py>, input_generator: PyObject) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        
        future_into_py(py, async move {
            let is_generator = Python::with_gil(|py| {
                input_generator.bind(py).hasattr("__next__").unwrap_or(false) && 
                input_generator.bind(py).hasattr("__iter__").unwrap_or(false)
            });
            
            if !is_generator {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Input must be a generator or iterator"));
            }
            
            let input_stream = stream! {
                loop {
                    let next_item = tokio::task::spawn_blocking({
                        let generator = Python::with_gil(|py| input_generator.clone_ref(py));
                        move || {
                            Python::with_gil(|py| {
                                match generator.call_method0(py, "__next__") {
                                    Ok(item) => {
                                        match item.extract::<PyPayload>(py) {
                                            Ok(payload) => Ok(Some(payload.to_rust())),
                                            Err(_) => Err(anyhow::anyhow!("Generator item must be Payload"))
                                        }
                                    },
                                    Err(e) => {
                                        if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) {
                                            Ok(None)
                                        } else {
                                            Err(anyhow::anyhow!("Generator error: {}", e))
                                        }
                                    }
                                }
                            })
                        }
                    }).await;

                    match next_item {
                        Ok(Ok(Some(payload))) => {
                            yield Ok(payload);
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        },
                        Ok(Ok(None)) => break,
                        Ok(Err(e)) => {
                            yield Err(e);
                            break;
                        },
                        Err(e) => {
                            yield Err(anyhow::anyhow!("Task join error: {}", e));
                            break;
                        }
                    }
                }
            };
            
            let mut response_stream = client.request_channel(Box::pin(input_stream));
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

    fn request_channel_reactive_streaming<'py>(&self, py: Python<'py>, input_generator: PyObject, on_response: PyObject, on_complete: Option<PyObject>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        
        future_into_py(py, async move {
            let is_generator = Python::with_gil(|py| {
                input_generator.bind(py).hasattr("__next__").unwrap_or(false) && 
                input_generator.bind(py).hasattr("__iter__").unwrap_or(false)
            });
            
            if !is_generator {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Input must be a generator or iterator"));
            }
            
            let input_stream = stream! {
                loop {
                    let next_item = tokio::task::spawn_blocking({
                        let generator = Python::with_gil(|py| input_generator.clone_ref(py));
                        move || {
                            Python::with_gil(|py| {
                                match generator.call_method0(py, "__next__") {
                                    Ok(item) => {
                                        match item.extract::<PyPayload>(py) {
                                            Ok(payload) => Ok(Some(payload.to_rust())),
                                            Err(_) => Err(anyhow::anyhow!("Generator item must be Payload"))
                                        }
                                    },
                                    Err(e) => {
                                        if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) {
                                            Ok(None)
                                        } else {
                                            Err(anyhow::anyhow!("Generator error: {}", e))
                                        }
                                    }
                                }
                            })
                        }
                    }).await;

                    match next_item {
                        Ok(Ok(Some(payload))) => {
                            yield Ok(payload);
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        },
                        Ok(Ok(None)) => break,
                        Ok(Err(e)) => {
                            yield Err(e);
                            break;
                        },
                        Err(e) => {
                            yield Err(anyhow::anyhow!("Task join error: {}", e));
                            break;
                        }
                    }
                }
            };
            
            let mut response_stream = client.request_channel(Box::pin(input_stream));
            let mut response_count = 0;
            let mut channel_error: Option<String> = None;
            
            while let Some(item) = response_stream.next().await {
                match item {
                    Ok(payload) => {
                        response_count += 1;
                        let py_payload = PyPayload::from_rust(payload);
                        
                        let callback_result = Python::with_gil(|py| {
                            on_response.call1(py, (py_payload, response_count))
                        });
                        
                        if let Err(e) = callback_result {
                            channel_error = Some(format!("Response callback error: {}", e));
                            break;
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    },
                    Err(e) => {
                        channel_error = Some(format!("Channel error: {}", e));
                        break;
                    }
                }
            }
            
            if let Some(on_complete_callback) = on_complete {
                let completion_result = Python::with_gil(|py| {
                    if let Some(ref error) = channel_error {
                        on_complete_callback.call1(py, (response_count, py.None(), error.clone()))
                    } else {
                        on_complete_callback.call1(py, (response_count, true, py.None()))
                    }
                });
                
                if let Err(e) = completion_result {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Completion callback error: {}", e)));
                }
            }
            
            if let Some(error) = channel_error {
                return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error));
            }
            
            Ok(response_count)
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
