use quinn::Endpoint;
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::ServerTransport};

use crate::{client::P2PClientTransport, connection::P2PConnection, misc::{create_p2p_endpoint, P2PConfig}};

#[derive(Debug)]
pub struct P2PServerTransport {
    config: P2PConfig,
    endpoint: Option<Endpoint>,
}

impl P2PServerTransport {
    fn new(config: P2PConfig) -> P2PServerTransport {
        P2PServerTransport {
            config,
            endpoint: None,
        }
    }
}

#[async_trait]
impl ServerTransport for P2PServerTransport {
    type Item = P2PClientTransport;

    async fn start(&mut self) -> rsocket_rust::Result<()> {
        if self.endpoint.is_some() {
            return Ok(());
        }
        
        let endpoint = create_p2p_endpoint(&self.config).await
            .map_err(|e| RSocketError::Other(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("P2P server endpoint creation failed: {}", e)
            ).into()))?;
        
        let local_addr = endpoint.local_addr()
            .map_err(|e| RSocketError::Other(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get local address: {}", e)
            ).into()))?;
        
        log::info!("P2P server started with PeerID: {}", self.config.peer_id);
        log::info!("P2P server listening on: {}", local_addr);
        
        self.endpoint = Some(endpoint);
        Ok(())
    }

    async fn next(&mut self) -> Option<rsocket_rust::Result<Self::Item>> {
        match self.endpoint.as_mut() {
            Some(endpoint) => {
                match endpoint.accept().await {
                    Some(connecting) => {
                        match connecting.await {
                            Ok(connection) => {
                                match connection.accept_bi().await {
                                    Ok((send_stream, recv_stream)) => {
                                        let p2p_connection = P2PConnection::new(send_stream, recv_stream);
                                        Some(Ok(P2PClientTransport::from_p2p_connection(p2p_connection)))
                                    }
                                    Err(e) => Some(Err(RSocketError::Other(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("P2P connection error: {}", e)
                                    ).into()).into())),
                                }
                            }
                            Err(e) => Some(Err(RSocketError::Other(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("P2P connection error: {}", e)
                            ).into()).into())),
                        }
                    }
                    None => None,
                }
            }
            None => None,
        }
    }
}

impl Default for P2PServerTransport {
    fn default() -> Self {
        P2PServerTransport::new(P2PConfig::default())
    }
}

impl From<P2PConfig> for P2PServerTransport {
    fn from(config: P2PConfig) -> Self {
        P2PServerTransport::new(config)
    }
}

impl From<String> for P2PServerTransport {
    fn from(addr: String) -> Self {
        let mut config = P2PConfig::default();
        config.listen_addr = addr.parse().expect("Invalid address format");
        P2PServerTransport::new(config)
    }
}

impl From<&str> for P2PServerTransport {
    fn from(addr: &str) -> Self {
        let mut config = P2PConfig::default();
        config.listen_addr = addr.parse().expect("Invalid address format");
        P2PServerTransport::new(config)
    }
}
