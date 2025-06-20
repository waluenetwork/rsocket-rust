use napi::bindgen_prelude::*;
use napi_derive::napi;
use rsocket_rust::prelude::*;
use crate::transport::JsTransportConfig;
use crate::payload::JsPayload;
use crate::get_runtime;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use async_trait::async_trait;

#[napi]
pub struct JsRSocketServer {
    server: Arc<Mutex<Option<JoinHandle<()>>>>,
    transport_config: JsTransportConfig,
    request_handler: Arc<Mutex<Option<ThreadsafeFunction<JsPayload, Promise<Option<JsPayload>>>>>>,
}

#[napi]
impl JsRSocketServer {
    #[napi(constructor)]
    pub fn new(transport_config: JsTransportConfig) -> Result<Self> {
        Ok(JsRSocketServer {
            server: Arc::new(Mutex::new(None)),
            transport_config,
            request_handler: Arc::new(Mutex::new(None)),
        })
    }
    
    #[napi]
    pub async fn set_request_handler(&self, handler: ThreadsafeFunction<JsPayload, Promise<Option<JsPayload>>>) -> Result<()> {
        let mut handler_guard = self.request_handler.lock().await;
        *handler_guard = Some(handler);
        Ok(())
    }
    
    #[napi]
    pub async fn start(&self) -> Result<()> {
        let transport_config = self.transport_config.clone();
        let handler = self.request_handler.clone();
        let server_arc = self.server.clone();
        
        let server_handle = tokio::spawn(async move {
            match transport_config.transport_type.transport_type.as_str() {
                "tcp" => {
                    let mut transport = rsocket_rust_transport_tcp::TcpServerTransport::from(transport_config.address.as_str());
                    transport.start().await?;
                    println!("TCP RSocket server started on {}", transport_config.address);
                    
                    while let Some(conn_result) = transport.next().await {
                        match conn_result {
                            Ok(_conn) => {
                                println!("New TCP connection accepted");
                            },
                            Err(e) => {
                                eprintln!("TCP connection error: {}", e);
                            }
                        }
                    }
                },
                "websocket" => {
                    let mut transport = rsocket_rust_transport_websocket::WebsocketServerTransport::from(transport_config.address.as_str());
                    transport.start().await?;
                    println!("WebSocket RSocket server started on {}", transport_config.address);
                    
                    while let Some(conn_result) = transport.next().await {
                        match conn_result {
                            Ok(_conn) => {
                                println!("New WebSocket connection accepted");
                            },
                            Err(e) => {
                                eprintln!("WebSocket connection error: {}", e);
                            }
                        }
                    }
                },
                _ => return Err(format!("Unsupported server transport type: {}", transport_config.transport_type.transport_type).into()),
            }
            ()
        });
        
        let mut server_guard = server_arc.lock().await;
        *server_guard = Some(server_handle);
        
        Ok(())
    }
    
    #[napi]
    pub async fn stop(&self) -> Result<()> {
        let server_arc = self.server.clone();
        let mut server_guard = server_arc.lock().await;
        if let Some(server) = server_guard.take() {
            server.abort();
        }
        Ok(())
    }
    
    #[napi]
    pub async fn is_running(&self) -> bool {
        let server_arc = self.server.clone();
        let server_guard = server_arc.lock().await;
        server_guard.is_some()
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

struct JsRequestHandler {
    handler: Arc<Mutex<Option<ThreadsafeFunction<JsPayload, Promise<Option<JsPayload>>>>>>,
}

impl JsRequestHandler {
    fn new(handler: Arc<Mutex<Option<ThreadsafeFunction<JsPayload, Promise<Option<JsPayload>>>>>>) -> Self {
        JsRequestHandler { handler }
    }
}

#[async_trait]
impl RSocket for JsRequestHandler {
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>, anyhow::Error> {
        let handler_arc = self.handler.clone();
        let handler_guard = handler_arc.lock().await;
        
        if let Some(handler) = handler_guard.as_ref() {
            let js_payload = JsPayload::from_rsocket_payload(req)
                .map_err(|e| anyhow::Error::msg(format!("Payload conversion failed: {}", e)))?;
            
            let result = handler.call_async(js_payload).await
                .map_err(|e| anyhow::Error::msg(format!("Handler call failed: {:?}", e)))?;
            
            match result {
                Some(js_result) => {
                    let rsocket_payload = js_result.to_rsocket_payload()
                        .map_err(|e| anyhow::Error::msg(format!("Payload conversion failed: {}", e)))?;
                    Ok(Some(rsocket_payload))
                },
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
    
    async fn fire_and_forget(&self, _req: Payload) -> Result<(), anyhow::Error> {
        Ok(())
    }
    
    async fn metadata_push(&self, _req: Payload) -> Result<(), anyhow::Error> {
        Ok(())
    }
    
    fn request_stream(&self, _req: Payload) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Payload, anyhow::Error>> + Send + 'static>> {
        use futures::stream;
        Box::pin(stream::empty())
    }
    
    fn request_channel(&self, _reqs: std::pin::Pin<Box<dyn futures::Stream<Item = Result<Payload, anyhow::Error>> + Send + 'static>>) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Payload, anyhow::Error>> + Send + 'static>> {
        use futures::stream;
        Box::pin(stream::empty())
    }
}
