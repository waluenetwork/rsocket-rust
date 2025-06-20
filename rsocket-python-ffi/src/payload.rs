use pyo3::prelude::*;
use rsocket_rust::prelude::*;

#[derive(Clone)]
#[pyclass(name = "Payload")]
pub struct PyPayload {
    inner: Payload,
}

#[pymethods]
impl PyPayload {
    #[staticmethod]
    pub fn builder() -> PyPayloadBuilder {
        PyPayloadBuilder {
            inner: Payload::builder(),
        }
    }
    
    pub fn data_utf8(&self) -> Option<String> {
        self.inner.data_utf8().map(|s| s.to_string())
    }
    
    pub fn metadata_utf8(&self) -> Option<String> {
        self.inner.metadata_utf8().map(|s| s.to_string())
    }
    
    pub fn data(&self) -> Option<Vec<u8>> {
        self.inner.data().map(|b| b.to_vec())
    }
    
    pub fn metadata(&self) -> Option<Vec<u8>> {
        self.inner.metadata().map(|b| b.to_vec())
    }
}

#[pyclass(name = "PayloadBuilder")]
pub struct PyPayloadBuilder {
    inner: PayloadBuilder,
}

#[pymethods]
impl PyPayloadBuilder {
    pub fn set_data_utf8(&mut self, data: &str) -> PyResult<()> {
        self.inner = std::mem::replace(&mut self.inner, Payload::builder()).set_data_utf8(data);
        Ok(())
    }
    
    pub fn set_metadata_utf8(&mut self, metadata: &str) -> PyResult<()> {
        self.inner = std::mem::replace(&mut self.inner, Payload::builder()).set_metadata_utf8(metadata);
        Ok(())
    }
    
    pub fn set_data(&mut self, data: Vec<u8>) -> PyResult<()> {
        self.inner = std::mem::replace(&mut self.inner, Payload::builder()).set_data(data);
        Ok(())
    }
    
    pub fn set_metadata(&mut self, metadata: Vec<u8>) -> PyResult<()> {
        self.inner = std::mem::replace(&mut self.inner, Payload::builder()).set_metadata(metadata);
        Ok(())
    }
    
    pub fn build(&mut self) -> PyPayload {
        let builder = std::mem::replace(&mut self.inner, Payload::builder());
        PyPayload {
            inner: builder.build(),
        }
    }
}

impl From<Payload> for PyPayload {
    fn from(payload: Payload) -> Self {
        PyPayload { inner: payload }
    }
}

impl From<PyPayload> for Payload {
    fn from(payload: PyPayload) -> Self {
        payload.inner
    }
}
