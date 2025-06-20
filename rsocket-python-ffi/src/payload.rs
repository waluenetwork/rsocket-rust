use pyo3::prelude::*;
use pyo3::types::PyBytes;
use bytes::Bytes;
use rsocket_rust::prelude::Payload;

#[pyclass]
#[derive(Clone)]
pub struct PyPayload {
    data: Option<Bytes>,
    metadata: Option<Bytes>,
}

#[pymethods]
impl PyPayload {
    #[new]
    pub fn new(data: Option<&PyBytes>, metadata: Option<&PyBytes>) -> PyResult<Self> {
        let data_bytes = data.map(|d| Bytes::copy_from_slice(d.as_bytes()));
        let metadata_bytes = metadata.map(|m| Bytes::copy_from_slice(m.as_bytes()));
        
        Ok(PyPayload {
            data: data_bytes,
            metadata: metadata_bytes,
        })
    }
    
    #[staticmethod]
    pub fn from_str(data: Option<String>, metadata: Option<String>) -> PyResult<Self> {
        let data_bytes = data.map(|d| Bytes::from(d));
        let metadata_bytes = metadata.map(|m| Bytes::from(m));
        
        Ok(PyPayload {
            data: data_bytes,
            metadata: metadata_bytes,
        })
    }
    
    pub fn get_data_utf8(&self) -> PyResult<Option<String>> {
        match &self.data {
            Some(data) => {
                let s = String::from_utf8(data.to_vec())
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyUnicodeDecodeError, _>(format!("Invalid UTF-8: {}", e)))?;
                Ok(Some(s))
            },
            None => Ok(None),
        }
    }
    
    pub fn get_metadata_utf8(&self) -> PyResult<Option<String>> {
        match &self.metadata {
            Some(metadata) => {
                let s = String::from_utf8(metadata.to_vec())
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyUnicodeDecodeError, _>(format!("Invalid UTF-8: {}", e)))?;
                Ok(Some(s))
            },
            None => Ok(None),
        }
    }
    
    pub fn get_data_bytes<'py>(&self, py: Python<'py>) -> PyResult<Option<&'py PyBytes>> {
        match &self.data {
            Some(data) => Ok(Some(PyBytes::new(py, data))),
            None => Ok(None),
        }
    }
    
    pub fn get_metadata_bytes<'py>(&self, py: Python<'py>) -> PyResult<Option<&'py PyBytes>> {
        match &self.metadata {
            Some(metadata) => Ok(Some(PyBytes::new(py, metadata))),
            None => Ok(None),
        }
    }
    
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
    
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }
    
    pub fn data_len(&self) -> usize {
        self.data.as_ref().map(|d| d.len()).unwrap_or(0)
    }
    
    pub fn metadata_len(&self) -> usize {
        self.metadata.as_ref().map(|m| m.len()).unwrap_or(0)
    }
    
    pub fn __str__(&self) -> String {
        let data_str = self.get_data_utf8().unwrap_or(None)
            .unwrap_or_else(|| format!("<{} bytes>", self.data_len()));
        let metadata_str = self.get_metadata_utf8().unwrap_or(None)
            .unwrap_or_else(|| format!("<{} bytes>", self.metadata_len()));
        
        format!("PyPayload(data='{}', metadata='{}')", data_str, metadata_str)
    }
    
    pub fn __repr__(&self) -> String {
        self.__str__()
    }
}

impl PyPayload {
    pub fn to_rsocket_payload(&self) -> PyResult<Payload> {
        let mut builder = Payload::builder();
        
        if let Some(data) = &self.data {
            builder = builder.set_data(data.clone());
        }
        
        if let Some(metadata) = &self.metadata {
            builder = builder.set_metadata(metadata.clone());
        }
        
        Ok(builder.build())
    }
    
    pub fn from_rsocket_payload(payload: Payload) -> PyResult<Self> {
        Ok(PyPayload {
            data: payload.data().map(|d| d.clone()),
            metadata: payload.metadata().map(|m| m.clone()),
        })
    }
}
