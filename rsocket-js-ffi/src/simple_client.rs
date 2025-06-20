use napi::bindgen_prelude::*;
use napi_derive::napi;
use rsocket_rust::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[napi]
pub struct SimpleRSocketClient {
    client: Arc<Mutex<Option<Box<dyn RSocket>>>>,
}

#[napi]
impl SimpleRSocketClient {
    #[napi(constructor)]
    pub fn new() -> Self {
        SimpleRSocketClient {
            client: Arc::new(Mutex::new(None)),
        }
    }
    
    #[napi]
    pub async fn connect_tcp(&self, address: String) -> Result<()> {
        let transport = rsocket_rust_transport_tcp::TcpClientTransport::from(address.as_str());
        let client = RSocketFactory::connect()
            .transport(transport)
            .start()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Connection failed: {}", e)))?;
        
        let mut client_guard = self.client.lock().await;
        *client_guard = Some(Box::new(client));
        
        Ok(())
    }
    
    #[napi]
    pub async fn connect_websocket(&self, address: String) -> Result<()> {
        let transport = rsocket_rust_transport_websocket::WebsocketClientTransport::from(address.as_str());
        let client = RSocketFactory::connect()
            .transport(transport)
            .start()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Connection failed: {}", e)))?;
        
        let mut client_guard = self.client.lock().await;
        *client_guard = Some(Box::new(client));
        
        Ok(())
    }
    
    #[napi]
    pub async fn request_response(&self, data: String) -> Result<Option<String>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Client not connected"))?;
        
        let payload = Payload::builder().set_data_utf8(&data).build();
        let result = client.request_response(payload).await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Request failed: {}", e)))?;
        
        match result {
            Some(payload) => Ok(payload.data_utf8().map(|s| s.to_string())),
            None => Ok(None),
        }
    }
    
    #[napi]
    pub async fn fire_and_forget(&self, data: String) -> Result<()> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "Client not connected"))?;
        
        let payload = Payload::builder().set_data_utf8(&data).build();
        client.fire_and_forget(payload).await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Fire and forget failed: {}", e)))?;
        
        Ok(())
    }
    
    #[napi]
    pub async fn close(&self) -> Result<()> {
        let mut client_guard = self.client.lock().await;
        *client_guard = None;
        Ok(())
    }
    
    #[napi]
    pub async fn is_connected(&self) -> bool {
        let client_guard = self.client.lock().await;
        client_guard.is_some()
    }
}
