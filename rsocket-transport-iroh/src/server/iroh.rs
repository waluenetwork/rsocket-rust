use iroh::protocol::{ProtocolHandler, Router};
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::ServerTransport, Result};
use futures::channel::mpsc;
use futures::future::BoxFuture;
use futures::StreamExt;
use anyhow;

use crate::{client::IrohClientTransport, misc::{create_iroh_endpoint, IrohConfig, RSOCKET_ALPN}};

#[derive(Debug)]
pub struct IrohServerTransport {
    config: IrohConfig,
    router: Option<Router>,
    connection_receiver: Option<mpsc::UnboundedReceiver<iroh::endpoint::Connection>>,
}

impl IrohServerTransport {
    fn new(config: IrohConfig) -> IrohServerTransport {
        IrohServerTransport {
            config,
            router: None,
            connection_receiver: None,
        }
    }
    
    pub fn node_id(&self) -> Option<String> {
        self.router.as_ref().map(|router| router.endpoint().node_id().to_string())
    }
    
    pub async fn node_addr(&self) -> Option<iroh::NodeAddr> {
        if let Some(router) = &self.router {
            let endpoint = router.endpoint();
            
            log::info!("Waiting for endpoint to discover direct addresses...");
            if let Err(e) = endpoint.direct_addresses().initialized().await {
                log::warn!("Failed to get direct addresses: {:?}", e);
            }
            
            log::info!("Waiting for home relay connection...");
            if let Err(e) = endpoint.home_relay().initialized().await {
                log::warn!("Failed to establish home relay: {:?}", e);
            }
            
            match endpoint.node_addr().await {
                Ok(node_addr) => {
                    log::info!("Complete NodeAddr created - NodeId: {}, Relay: {:?}, Direct addresses: {:?}", 
                              node_addr.node_id, node_addr.relay_url, node_addr.direct_addresses);
                    Some(node_addr)
                },
                Err(e) => {
                    log::error!("Failed to get node address: {:?}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    pub async fn node_addr_string(&self) -> Option<String> {
        if let Some(node_addr) = self.node_addr().await {
            Some(format!("{:?}", node_addr))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
struct RSocketProtocolHandler {
    connection_sender: mpsc::UnboundedSender<iroh::endpoint::Connection>,
}

impl ProtocolHandler for RSocketProtocolHandler {
    fn accept(&self, connection: iroh::endpoint::Connection) -> BoxFuture<'static, anyhow::Result<()>> {
        let sender = self.connection_sender.clone();
        Box::pin(async move {
            sender.unbounded_send(connection).map_err(|e| anyhow::anyhow!("Failed to send connection: {}", e))?;
            Ok(())
        })
    }
}

#[async_trait]
impl ServerTransport for IrohServerTransport {
    type Item = IrohClientTransport;

    async fn start(&mut self) -> Result<()> {
        if self.router.is_some() {
            return Ok(());
        }
        
        let endpoint = create_iroh_endpoint(&self.config).await
            .map_err(|e| RSocketError::Other(anyhow::anyhow!("Failed to create endpoint: {}", e).into()))?;
        
        let (connection_sender, connection_receiver) = mpsc::unbounded();
        let protocol_handler = RSocketProtocolHandler { connection_sender };
        
        let router = Router::builder(endpoint)
            .accept(RSOCKET_ALPN, protocol_handler)
            .spawn()
            .await
            .map_err(|e| RSocketError::Other(anyhow::anyhow!("Failed to start router: {}", e).into()))?;
        
        log::info!("Iroh P2P server started with NodeId: {}", router.endpoint().node_id());
        log::info!("Server listening for P2P connections...");
        
        self.router = Some(router);
        self.connection_receiver = Some(connection_receiver);
        Ok(())
    }

    async fn next(&mut self) -> Option<Result<Self::Item>> {
        match self.connection_receiver.as_mut() {
            Some(receiver) => {
                match receiver.next().await {
                    Some(connection) => {
                        log::info!("✅ Server: Received incoming Iroh P2P connection");
                        
                        match connection.accept_bi().await {
                            Ok((send_stream, recv_stream)) => {
                                log::info!("✅ Server: Opened bidirectional stream for incoming connection");
                                let connection_with_streams = crate::connection::IrohConnectionWithStreams::new(send_stream, recv_stream);
                                Some(Ok(IrohClientTransport::from_connection_with_streams(connection_with_streams)))
                            }
                            Err(e) => {
                                log::error!("❌ Server: Failed to open bidirectional stream: {:?}", e);
                                Some(Err(RSocketError::Other(anyhow::anyhow!("Failed to open bidirectional stream: {}", e).into()).into()))
                            }
                        }
                    }
                    None => {
                        log::warn!("❌ Server: Connection receiver closed");
                        None
                    }
                }
            }
            None => {
                log::warn!("❌ Server: No connection receiver available");
                None
            }
        }
    }
}

impl Default for IrohServerTransport {
    fn default() -> Self {
        IrohServerTransport::new(IrohConfig::default())
    }
}

impl From<IrohConfig> for IrohServerTransport {
    fn from(config: IrohConfig) -> Self {
        IrohServerTransport::new(config)
    }
}
