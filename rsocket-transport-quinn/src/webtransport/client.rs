use rsocket_rust::async_trait;
use rsocket_rust::{transport::Transport, Result, error::RSocketError};

use super::connection::WebTransportConnection;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::WebTransport;

#[derive(Debug)]
pub enum WebTransportClientTransport {
    #[cfg(not(target_arch = "wasm32"))]
    Native(crate::client::QuinnClientTransport),
    #[cfg(target_arch = "wasm32")]
    Browser { url: String },
}

#[async_trait]
impl Transport for WebTransportClientTransport {
    type Conn = WebTransportConnection;

    async fn connect(self) -> Result<WebTransportConnection> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            WebTransportClientTransport::Native(quinn_transport) => {
                let quinn_conn = quinn_transport.connect().await?;
                Ok(WebTransportConnection::from_quinn(quinn_conn))
            }
            #[cfg(target_arch = "wasm32")]
            WebTransportClientTransport::Browser { url } => {
                let transport = WebTransport::new(&url)
                    .map_err(|e| RSocketError::Other(format!("Failed to create WebTransport: {:?}", e).into()))?;
                
                let ready_promise = transport.ready();
                wasm_bindgen_futures::JsFuture::from(ready_promise)
                    .await
                    .map_err(|e| RSocketError::Other(format!("WebTransport ready failed: {:?}", e).into()))?;

                let stream = transport.create_bidirectional_stream()
                    .map_err(|e| RSocketError::Other(format!("Failed to create bidirectional stream: {:?}", e).into()))?;
                
                let stream_promise = wasm_bindgen_futures::JsFuture::from(stream).await
                    .map_err(|e| RSocketError::Other(format!("Stream creation failed: {:?}", e).into()))?;

                let readable = js_sys::Reflect::get(&stream_promise, &"readable".into())
                    .map_err(|e| RSocketError::Other(format!("Failed to get readable stream: {:?}", e).into()))?;
                let writable = js_sys::Reflect::get(&stream_promise, &"writable".into())
                    .map_err(|e| RSocketError::Other(format!("Failed to get writable stream: {:?}", e).into()))?;

                let readable_stream = readable.dyn_into::<web_sys::ReadableStream>()
                    .map_err(|e| RSocketError::Other(format!("Failed to cast to ReadableStream: {:?}", e).into()))?;
                let writable_stream = writable.dyn_into::<web_sys::WritableStream>()
                    .map_err(|e| RSocketError::Other(format!("Failed to cast to WritableStream: {:?}", e).into()))?;

                Ok(WebTransportConnection::from_streams(readable_stream, writable_stream))
            }
        }
    }
}

impl From<String> for WebTransportClientTransport {
    fn from(addr: String) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            if addr.starts_with("https://") || addr.starts_with("wss://") {
                Self::Browser { url: addr }
            } else {
                panic!("WebTransport in browser requires HTTPS URL")
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::Native(crate::client::QuinnClientTransport::from(addr))
        }
    }
}

impl From<&str> for WebTransportClientTransport {
    fn from(addr: &str) -> Self {
        Self::from(addr.to_string())
    }
}

impl WebTransportClientTransport {
    pub fn new_browser(_url: String) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::Browser { url: _url }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            panic!("Browser WebTransport is only available on WASM target")
        }
    }

    pub fn new_native(addr: String) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::Native(crate::client::QuinnClientTransport::from(addr))
        }
        #[cfg(target_arch = "wasm32")]
        {
            panic!("Native Quinn transport is not available on WASM target")
        }
    }
}
