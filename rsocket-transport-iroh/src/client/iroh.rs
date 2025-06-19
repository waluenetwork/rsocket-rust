
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::Transport};

use crate::{connection::P2PConnection, misc::{create_p2p_endpoint, parse_peer_address, P2PConfig}};

#[derive(Debug)]
enum Connector {
    Direct(P2PConnection),
    Lazy(String, Option<P2PConfig>),
}

#[derive(Debug)]
pub struct P2PClientTransport {
    connector: Connector,
}

#[async_trait]
impl Transport for P2PClientTransport {
    type Conn = P2PConnection;

    async fn connect(self) -> rsocket_rust::Result<P2PConnection> {
        match self.connector {
            Connector::Direct(p2p_connection) => {
                Ok(p2p_connection)
            }
            Connector::Lazy(addr, config) => {
                let config = config.unwrap_or_default();
                let endpoint = create_p2p_endpoint(&config).await
                    .map_err(|e| RSocketError::Other(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("P2P endpoint creation failed: {}", e)
                    ).into()))?;
                
                let (_peer_id, socket_addr) = parse_peer_address(&addr)?;
                
                let connection = endpoint.connect(socket_addr, "localhost")
                    .map_err(|e| RSocketError::Other(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("P2P connection failed: {}", e)
                    ).into()))?
                    .await
                    .map_err(|e| RSocketError::Other(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("P2P connection await failed: {}", e)
                    ).into()))?;
                
                P2PConnection::from_quinn_connection(connection).await
            }
        }
    }
}

impl From<String> for P2PClientTransport {
    fn from(addr: String) -> Self {
        P2PClientTransport {
            connector: Connector::Lazy(addr, None),
        }
    }
}

impl From<&str> for P2PClientTransport {
    fn from(addr: &str) -> Self {
        P2PClientTransport {
            connector: Connector::Lazy(addr.to_string(), None),
        }
    }
}

impl P2PClientTransport {
    pub fn from_p2p_connection(p2p_connection: P2PConnection) -> Self {
        P2PClientTransport {
            connector: Connector::Direct(p2p_connection),
        }
    }
    
    pub fn with_config(addr: String, config: P2PConfig) -> Self {
        P2PClientTransport {
            connector: Connector::Lazy(addr, Some(config)),
        }
    }
}

impl From<P2PConnection> for P2PClientTransport {
    fn from(p2p_connection: P2PConnection) -> Self {
        P2PClientTransport {
            connector: Connector::Direct(p2p_connection),
        }
    }
}
