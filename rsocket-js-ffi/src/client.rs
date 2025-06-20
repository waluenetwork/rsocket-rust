use napi::bindgen_prelude::*;
use napi_derive::napi;
use rsocket_rust::prelude::*;
use crate::transport::JsTransportConfig;
use crate::payload::JsPayload;
use crate::get_runtime;
use std::sync::Arc;
use tokio::sync::Mutex;

#[napi]
pub struct JsRSocketClient {
    client: Arc<Mutex<Option<Box<dyn RSocket>>>>,
    transport_config: JsTransportConfig,
}

#[napi]
impl JsRSocketClient {
    #[napi(constructor)]
    pub fn new(transport_config: JsTransportConfig) -> Result<Self> {
        Ok(JsRSocketClient {
            client: Arc::new(Mutex::new(None)),
            transport_config,
        })
    }
    
    #[napi]
    pub async fn connect(&self) -> Result<()> {
        let transport_config = self.transport_config.clone();
        let client_arc = self.client.clone();
        
        let client = match transport_config.transport_type.transport_type.as_str() {
            "tcp" => {
                let transport = rsocket_rust_transport_tcp::TcpClientTransport::from(transport_config.address.as_str());
                let client = RSocketFactory::connect()
                    .transport(transport)
                    .start()
                    .await
                    .map_err(|e| Error::new(Status::GenericFailure, format!("TCP connection failed: {}", e)))?;
                Box::new(client) as Box<dyn RSocket>
            },
            "websocket" => {
                let transport = rsocket_rust_transport_websocket::WebsocketClientTransport::from(transport_config.address.as_str());
                let client = RSocketFactory::connect()
                    .transport(transport)
                    .start()
                    .await
                    .map_err(|e| Error::new(Status::GenericFailure, format!("WebSocket connection failed: {}", e)))?;
                Box::new(client) as Box<dyn RSocket>
            },
            "quinn-quic" => {
                return Err(Error::new(Status::InvalidArg, "QUIC transport not available in current build"));
            },
            _ => return Err(Error::new(Status::InvalidArg, format!("Unsupported transport type: {}", transport_config.transport_type.transport_type))),
        };
        
        let mut client_guard = client_arc.lock().await;
        *client_guard = Some(client);
        
        Ok(())
    }
    
    #[napi]
    pub async fn request_response(&self, payload: &JsPayload) -> Result<Option<JsPayload>> {
        let client_arc = self.client.clone();
        let client_guard = client_arc.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Client not connected"))?;
        
        let rsocket_payload = payload.to_rsocket_payload()?;
        let result = client.request_response(rsocket_payload).await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Request failed: {}", e)))?;
        
        match result {
            Some(payload) => Ok(Some(JsPayload::from_rsocket_payload(payload)?)),
            None => Ok(None),
        }
    }
    
    #[napi]
    pub async fn request_stream(&self, payload: &JsPayload) -> Result<Vec<JsPayload>> {
        let client_arc = self.client.clone();
        let client_guard = client_arc.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Client not connected"))?;
        
        let rsocket_payload = payload.to_rsocket_payload()?;
        let mut stream = client.request_stream(rsocket_payload);
        
        let mut results = Vec::new();
        use futures::StreamExt;
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(payload) => results.push(JsPayload::from_rsocket_payload(payload)?),
                Err(e) => return Err(Error::new(Status::GenericFailure, format!("Stream error: {}", e))),
            }
        }
        
        Ok(results)
    }
    
    #[napi]
    pub async fn fire_and_forget(&self, payload: &JsPayload) -> Result<()> {
        let client_arc = self.client.clone();
        let client_guard = client_arc.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Client not connected"))?;
        
        let rsocket_payload = payload.to_rsocket_payload()?;
        client.fire_and_forget(rsocket_payload).await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Fire and forget failed: {}", e)))?;
        
        Ok(())
    }
    
    #[napi]
    pub async fn request_channel(&self, payloads: Vec<&JsPayload>) -> Result<Vec<JsPayload>> {
        let client_arc = self.client.clone();
        let client_guard = client_arc.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Client not connected"))?;
        
        let rsocket_payloads: Result<Vec<_>> = payloads.into_iter()
            .map(|p| p.to_rsocket_payload())
            .collect();
        let rsocket_payloads = rsocket_payloads?;
        
        use futures::{StreamExt, stream};
        let input_stream = stream::iter(rsocket_payloads.into_iter().map(Ok));
        let mut output_stream = client.request_channel(Box::pin(input_stream));
        
        let mut results = Vec::new();
        while let Some(result) = output_stream.next().await {
            match result {
                Ok(payload) => results.push(JsPayload::from_rsocket_payload(payload)?),
                Err(e) => return Err(Error::new(Status::GenericFailure, format!("Channel error: {}", e))),
            }
        }
        
        Ok(results)
    }
    
    #[napi]
    pub async fn close(&self) -> Result<()> {
        let client_arc = self.client.clone();
        let mut client_guard = client_arc.lock().await;
        *client_guard = None;
        Ok(())
    }
    
    #[napi]
    pub async fn is_connected(&self) -> bool {
        let client_arc = self.client.clone();
        let client_guard = client_arc.lock().await;
        client_guard.is_some()
    }
    
    #[napi]
    pub fn get_transport_type(&self) -> String {
        self.transport_config.transport_type.transport_type.clone()
    }
    
    #[napi]
    pub fn get_address(&self) -> String {
        self.transport_config.address.clone()
    }
}
