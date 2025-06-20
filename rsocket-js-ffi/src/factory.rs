use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::transport::{JsTransportConfig, JsTransportType};
use crate::client::JsRSocketClient;
use crate::server::JsRSocketServer;

#[napi]
pub struct JsRSocketFactory;

#[napi]
impl JsRSocketFactory {
    #[napi(constructor)]
    pub fn new() -> Self {
        JsRSocketFactory
    }
    
    #[napi(factory)]
    pub fn create_tcp_client(address: String) -> Result<JsRSocketClient> {
        let config = JsTransportConfig::new(
            JsTransportType::tcp(),
            address,
            None,
        );
        JsRSocketClient::new(config)
    }
    
    #[napi(factory)]
    pub fn create_websocket_client(address: String) -> Result<JsRSocketClient> {
        let config = JsTransportConfig::new(
            JsTransportType::websocket(),
            address,
            None,
        );
        JsRSocketClient::new(config)
    }
    
    #[napi(factory)]
    pub fn create_quic_client(address: String) -> Result<JsRSocketClient> {
        let config = JsTransportConfig::new(
            JsTransportType::quinn_quic(),
            address,
            None,
        );
        JsRSocketClient::new(config)
    }
    
    #[napi(factory)]
    pub fn create_webtransport_client(address: String) -> Result<JsRSocketClient> {
        let config = JsTransportConfig::new(
            JsTransportType::quinn_webtransport(),
            address,
            None,
        );
        JsRSocketClient::new(config)
    }
    
    #[napi(factory)]
    pub fn create_optimized_client(transport_type: String, address: String, enable_simd: Option<bool>, enable_webworkers: Option<bool>) -> Result<JsRSocketClient> {
        let transport_type = match transport_type.as_str() {
            "tcp" => JsTransportType::tcp(),
            "websocket" => JsTransportType::websocket(),
            "quinn-quic" => JsTransportType::quinn_quic(),
            "quinn-webtransport" => JsTransportType::quinn_webtransport(),
            "iroh-roq" => JsTransportType::iroh_roq(),
            "wasm-webworkers" => JsTransportType::wasm_webworkers(),
            "iroh-p2p" => JsTransportType::iroh_p2p(),
            "iroh-p2p-wasm" => JsTransportType::iroh_p2p_wasm(),
            _ => return Err(Error::new(Status::InvalidArg, format!("Unsupported transport type: {}", transport_type))),
        };
        
        let mut config = JsTransportConfig::new(transport_type, address, None);
        
        if enable_simd.unwrap_or(true) {
            config.enable_simd_processing();
        }
        
        if enable_webworkers.unwrap_or(false) {
            config.enable_webworkers();
            config.set_webworkers_count(4);
        }
        
        config.set_performance_mode("high".to_string());
        config.enable_crossbeam_optimizations();
        
        JsRSocketClient::new(config)
    }
    
    #[napi(factory)]
    pub fn create_tcp_server(address: String) -> Result<JsRSocketServer> {
        let config = JsTransportConfig::new(
            JsTransportType::tcp(),
            address,
            None,
        );
        JsRSocketServer::new(config)
    }
    
    #[napi(factory)]
    pub fn create_websocket_server(address: String) -> Result<JsRSocketServer> {
        let config = JsTransportConfig::new(
            JsTransportType::websocket(),
            address,
            None,
        );
        JsRSocketServer::new(config)
    }
    
    #[napi(factory)]
    pub fn create_quic_server(address: String) -> Result<JsRSocketServer> {
        let config = JsTransportConfig::new(
            JsTransportType::quinn_quic(),
            address,
            None,
        );
        JsRSocketServer::new(config)
    }
    
    #[napi(factory)]
    pub fn create_high_performance_server(transport_type: String, address: String, worker_count: Option<u32>) -> Result<JsRSocketServer> {
        let transport_type = match transport_type.as_str() {
            "tcp" => JsTransportType::tcp(),
            "websocket" => JsTransportType::websocket(),
            "quinn-quic" => JsTransportType::quinn_quic(),
            _ => return Err(Error::new(Status::InvalidArg, format!("Unsupported server transport type: {}", transport_type))),
        };
        
        let mut config = JsTransportConfig::new(transport_type, address, None);
        config.set_performance_mode("high".to_string());
        config.enable_crossbeam_optimizations();
        config.enable_simd_processing();
        
        if let Some(count) = worker_count {
            config.set_webworkers_count(count);
        }
        
        JsRSocketServer::new(config)
    }
}
