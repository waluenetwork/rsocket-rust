use napi::bindgen_prelude::*;
use napi_derive::napi;
use rsocket_rust::prelude::Payload;
use bytes::Bytes;

#[napi(object)]
#[derive(Clone)]
pub struct JsPayload {
    inner: Payload,
}

#[napi]
impl JsPayload {
    #[napi(constructor)]
    pub fn new(data: Option<Buffer>, metadata: Option<Buffer>) -> Result<Self> {
        let data_bytes = data.map(|buf| Bytes::from(buf.to_vec()));
        let metadata_bytes = metadata.map(|buf| Bytes::from(buf.to_vec()));
        
        let payload = match (data_bytes, metadata_bytes) {
            (Some(data), Some(metadata)) => Payload::builder().set_data(data).set_metadata(metadata).build(),
            (Some(data), None) => Payload::builder().set_data(data).build(),
            (None, Some(metadata)) => Payload::builder().set_metadata(metadata).build(),
            (None, None) => Payload::builder().build(),
        };
        
        Ok(JsPayload { inner: payload })
    }
    
    #[napi]
    pub fn get_data(&self) -> Option<Buffer> {
        self.inner.data().map(|data| Buffer::from(data.to_vec()))
    }
    
    #[napi]
    pub fn get_metadata(&self) -> Option<Buffer> {
        self.inner.metadata().map(|metadata| Buffer::from(metadata.to_vec()))
    }
    
    #[napi]
    pub fn get_data_utf8(&self) -> Option<String> {
        self.inner.data_utf8().map(|s| s.to_string())
    }
    
    #[napi]
    pub fn get_metadata_utf8(&self) -> Option<String> {
        self.inner.metadata_utf8().map(|s| s.to_string())
    }
    
    #[napi]
    pub fn has_data(&self) -> bool {
        self.inner.data().is_some()
    }
    
    #[napi]
    pub fn has_metadata(&self) -> bool {
        self.inner.metadata().is_some()
    }
    
    #[napi]
    pub fn data_len(&self) -> u32 {
        self.inner.data().map_or(0, |data| data.len() as u32)
    }
    
    #[napi]
    pub fn metadata_len(&self) -> u32 {
        self.inner.metadata().map_or(0, |metadata| metadata.len() as u32)
    }
    
    #[napi]
    pub fn to_string(&self) -> String {
        format!("JsPayload(data: {}, metadata: {})", 
                self.data_len(), self.metadata_len())
    }
    
    #[napi(factory)]
    pub fn from_string(data: String, metadata: Option<String>) -> Result<Self> {
        let data_buffer = Buffer::from(data.into_bytes());
        let metadata_buffer = metadata.map(|m| Buffer::from(m.into_bytes()));
        Self::new(Some(data_buffer), metadata_buffer)
    }
    
    #[napi(factory)]
    pub fn from_json(data: String, metadata: Option<String>) -> Result<Self> {
        let data_buffer = Buffer::from(data.into_bytes());
        let metadata_buffer = metadata.map(|m| Buffer::from(m.into_bytes()));
        Self::new(Some(data_buffer), metadata_buffer)
    }
}

impl JsPayload {
    pub fn to_rsocket_payload(&self) -> Result<Payload> {
        Ok(self.inner.clone())
    }
    
    pub fn from_rsocket_payload(payload: Payload) -> Result<Self> {
        Ok(JsPayload { inner: payload })
    }
}
