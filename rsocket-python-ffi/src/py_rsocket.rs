use pyo3::prelude::*;
use async_trait::async_trait;
use async_stream::stream;
use rsocket_rust::prelude::{RSocket, Payload};
use rsocket_rust::Result;
use crate::payload::PyPayload;
use futures::StreamExt;
use std::pin::Pin;
use futures::Stream;

#[pyclass(name = "RSocketHandler")]
pub struct PyRSocketHandler {
    metadata_push_handler: Option<PyObject>,
    fire_and_forget_handler: Option<PyObject>,
    request_response_handler: Option<PyObject>,
    request_stream_handler: Option<PyObject>,
    request_channel_handler: Option<PyObject>,
}

impl Clone for PyRSocketHandler {
    fn clone(&self) -> Self {
        Python::with_gil(|py| {
            PyRSocketHandler {
                metadata_push_handler: self.metadata_push_handler.as_ref().map(|h| h.clone_ref(py)),
                fire_and_forget_handler: self.fire_and_forget_handler.as_ref().map(|h| h.clone_ref(py)),
                request_response_handler: self.request_response_handler.as_ref().map(|h| h.clone_ref(py)),
                request_stream_handler: self.request_stream_handler.as_ref().map(|h| h.clone_ref(py)),
                request_channel_handler: self.request_channel_handler.as_ref().map(|h| h.clone_ref(py)),
            }
        })
    }
}

#[pymethods]
impl PyRSocketHandler {
    #[new]
    pub fn new() -> Self {
        PyRSocketHandler {
            metadata_push_handler: None,
            fire_and_forget_handler: None,
            request_response_handler: None,
            request_stream_handler: None,
            request_channel_handler: None,
        }
    }

    pub fn metadata_push(mut self_: PyRefMut<Self>, handler: PyObject) -> PyRefMut<Self> {
        self_.metadata_push_handler = Some(handler);
        self_
    }

    pub fn fire_and_forget(mut self_: PyRefMut<Self>, handler: PyObject) -> PyRefMut<Self> {
        self_.fire_and_forget_handler = Some(handler);
        self_
    }

    pub fn request_response(mut self_: PyRefMut<Self>, handler: PyObject) -> PyRefMut<Self> {
        self_.request_response_handler = Some(handler);
        self_
    }

    pub fn request_stream(mut self_: PyRefMut<Self>, handler: PyObject) -> PyRefMut<Self> {
        self_.request_stream_handler = Some(handler);
        self_
    }

    pub fn request_channel(mut self_: PyRefMut<Self>, handler: PyObject) -> PyRefMut<Self> {
        self_.request_channel_handler = Some(handler);
        self_
    }

    fn __str__(&self) -> String {
        "RSocketHandler".to_string()
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

pub struct PyRSocket {
    handler: PyRSocketHandler,
}

impl PyRSocket {
    pub fn new(handler: PyRSocketHandler) -> Self {
        PyRSocket { handler }
    }
}

#[async_trait]
impl RSocket for PyRSocket {
    async fn metadata_push(&self, req: Payload) -> Result<()> {
        if let Some(ref handler) = self.handler.metadata_push_handler {
            let handler_clone = Python::with_gil(|py| handler.clone_ref(py));
            let py_payload = PyPayload::from_rust(req);
            
            let result = tokio::task::spawn_blocking(move || {
                Python::with_gil(|py| {
                    let result = handler_clone.call1(py, (py_payload,));
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(anyhow::anyhow!("Python handler error: {}", e))
                    }
                })
            }).await;
            
            match result {
                Ok(inner_result) => inner_result,
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e))
            }
        } else {
            Ok(())
        }
    }

    async fn fire_and_forget(&self, req: Payload) -> Result<()> {
        if let Some(ref handler) = self.handler.fire_and_forget_handler {
            let handler_clone = Python::with_gil(|py| handler.clone_ref(py));
            let py_payload = PyPayload::from_rust(req);
            
            let result = tokio::task::spawn_blocking(move || {
                Python::with_gil(|py| {
                    let result = handler_clone.call1(py, (py_payload,));
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(anyhow::anyhow!("Python handler error: {}", e))
                    }
                })
            }).await;
            
            match result {
                Ok(inner_result) => inner_result,
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e))
            }
        } else {
            Ok(())
        }
    }

    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        if let Some(ref handler) = self.handler.request_response_handler {
            let handler_clone = Python::with_gil(|py| handler.clone_ref(py));
            let py_payload = PyPayload::from_rust(req.clone());
            
            let result = tokio::task::spawn_blocking(move || {
                Python::with_gil(|py| {
                    let result = handler_clone.call1(py, (py_payload,));
                    match result {
                        Ok(py_result) => {
                            if py_result.is_none(py) {
                                Ok(None)
                            } else {
                                match py_result.extract::<PyPayload>(py) {
                                    Ok(payload) => Ok(Some(payload.to_rust())),
                                    Err(e) => Err(anyhow::anyhow!("Failed to extract payload: {}", e))
                                }
                            }
                        },
                        Err(e) => Err(anyhow::anyhow!("Python handler error: {}", e))
                    }
                })
            }).await;
            
            match result {
                Ok(inner_result) => inner_result,
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e))
            }
        } else {
            Ok(Some(req))
        }
    }

    fn request_stream(&self, req: Payload) -> Pin<Box<dyn Send + Stream<Item = Result<Payload>>>> {
        if let Some(ref handler) = self.handler.request_stream_handler {
            let handler_clone = Python::with_gil(|py| handler.clone_ref(py));
            let req_clone = req.clone();
            Box::pin(stream! {
                let result = Python::with_gil(|py| {
                    let py_payload = PyPayload::from_rust(req_clone);
                    let result = handler_clone.call1(py, (py_payload,));
                    match result {
                        Ok(py_result) => {
                            match py_result.extract::<Vec<PyPayload>>(py) {
                                Ok(payloads) => Ok(payloads),
                                Err(e) => Err(anyhow::anyhow!("Failed to extract payloads: {}", e))
                            }
                        },
                        Err(e) => Err(anyhow::anyhow!("Python handler error: {}", e))
                    }
                });

                match result {
                    Ok(py_results) => {
                        for py_payload in py_results {
                            yield Ok(py_payload.to_rust());
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                    }
                }
            })
        } else {
            Box::pin(stream! {
                yield Ok(req);
            })
        }
    }

    fn request_channel(&self, mut reqs: Pin<Box<dyn Send + Stream<Item = Result<Payload>>>>) -> Pin<Box<dyn Send + Stream<Item = Result<Payload>>>> {
        if let Some(ref handler) = self.handler.request_channel_handler {
            let handler_clone = Python::with_gil(|py| handler.clone_ref(py));
            Box::pin(stream! {
                let mut payloads = Vec::new();
                while let Some(item) = reqs.next().await {
                    match item {
                        Ok(payload) => payloads.push(payload),
                        Err(e) => {
                            yield Err(e);
                            return;
                        }
                    }
                }

                let result = Python::with_gil(|py| {
                    let py_payloads: Vec<PyPayload> = payloads.into_iter().map(PyPayload::from_rust).collect();
                    let result = handler_clone.call1(py, (py_payloads,));
                    match result {
                        Ok(py_result) => {
                            match py_result.extract::<Vec<PyPayload>>(py) {
                                Ok(payloads) => Ok(payloads),
                                Err(e) => Err(anyhow::anyhow!("Failed to extract payloads: {}", e))
                            }
                        },
                        Err(e) => Err(anyhow::anyhow!("Python handler error: {}", e))
                    }
                });

                match result {
                    Ok(py_results) => {
                        for py_payload in py_results {
                            yield Ok(py_payload.to_rust());
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                    }
                }
            })
        } else {
            reqs
        }
    }
}
