use rsocket_rust::async_trait;
use rsocket_rust::{transport::ServerTransport, Result};

use super::client::WebTransportClientTransport;

#[derive(Debug)]
pub enum WebTransportServerTransport {
    #[cfg(not(target_arch = "wasm32"))]
    Native(crate::server::QuinnServerTransport),
    #[cfg(target_arch = "wasm32")]
    Browser,
}

#[async_trait]
impl ServerTransport for WebTransportServerTransport {
    type Item = WebTransportClientTransport;

    async fn start(&mut self) -> Result<()> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            WebTransportServerTransport::Native(quinn_server) => {
                quinn_server.start().await
            }
            #[cfg(target_arch = "wasm32")]
            WebTransportServerTransport::Browser => {
                Err(RSocketError::Other("WebTransport server not supported in browser".into()).into())
            }
        }
    }

    async fn next(&mut self) -> Option<Result<Self::Item>> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            WebTransportServerTransport::Native(quinn_server) => {
                match quinn_server.next().await {
                    Some(Ok(quinn_client)) => {
                        Some(Ok(WebTransportClientTransport::Native(quinn_client)))
                    }
                    Some(Err(e)) => Some(Err(e)),
                    None => None,
                }
            }
            #[cfg(target_arch = "wasm32")]
            WebTransportServerTransport::Browser => None,
        }
    }
}

impl From<std::net::SocketAddr> for WebTransportServerTransport {
    fn from(addr: std::net::SocketAddr) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::Native(crate::server::QuinnServerTransport::from(addr))
        }
        #[cfg(target_arch = "wasm32")]
        {
            panic!("Native server transport is not available on WASM target")
        }
    }
}

impl From<String> for WebTransportServerTransport {
    fn from(addr: String) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::Native(crate::server::QuinnServerTransport::from(addr))
        }
        #[cfg(target_arch = "wasm32")]
        {
            panic!("Native server transport is not available on WASM target")
        }
    }
}

impl From<&str> for WebTransportServerTransport {
    fn from(addr: &str) -> Self {
        Self::from(addr.to_string())
    }
}
