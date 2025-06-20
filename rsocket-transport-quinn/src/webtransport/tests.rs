#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::webtransport::{WebTransportClientTransport, WebTransportServerTransport};

    #[test]
    fn test_webtransport_client_creation() {
        let client = WebTransportClientTransport::from("https://example.com:4433");
        
        #[cfg(target_arch = "wasm32")]
        {
            match client {
                WebTransportClientTransport::Browser { url } => {
                    assert_eq!(url, "https://example.com:4433");
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            match client {
                WebTransportClientTransport::Native(_) => {
                }
            }
        }
    }

    #[test]
    fn test_webtransport_server_creation() {
        use std::net::SocketAddr;
        
        let addr: SocketAddr = "127.0.0.1:4433".parse().unwrap();
        let server = WebTransportServerTransport::from(addr);
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            match server {
                WebTransportServerTransport::Native(_) => {
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_browser_webtransport_url_validation() {
        let client = WebTransportClientTransport::from("https://example.com:4433");
        match client {
            WebTransportClientTransport::Browser { url } => {
                assert_eq!(url, "https://example.com:4433");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_native_webtransport_fallback() {
        let client = WebTransportClientTransport::from("127.0.0.1:4433");
        match client {
            WebTransportClientTransport::Native(_) => {
            }
        }
    }
}
