
use iroh::{Endpoint, NodeAddr, NodeId};
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::Transport, Result};
use anyhow;

use crate::{connection::IrohConnectionWithStreams, misc::{create_iroh_endpoint, parse_node_addr, IrohConfig, RSOCKET_ALPN}};

#[derive(Debug)]
enum Connector {
    Direct(iroh::endpoint::Connection),
    DirectWithStreams(IrohConnectionWithStreams),
    Lazy(String),
    NodeAddr(iroh::NodeAddr),
}

#[derive(Debug)]
pub struct IrohClientTransport {
    connector: Connector,
}

#[async_trait]
impl Transport for IrohClientTransport {
    type Conn = IrohConnectionWithStreams;

    async fn connect(self) -> Result<IrohConnectionWithStreams> {
        match self.connector {
            Connector::Direct(connection) => {
                log::info!("🔗 Opening bidirectional stream for direct Iroh connection");
                let (send_stream, recv_stream) = connection.open_bi()
                    .await
                    .map_err(|e| RSocketError::Other(e.into()))?;
                
                log::info!("✅ Bidirectional stream opened successfully");
                Ok(IrohConnectionWithStreams::new(send_stream, recv_stream))
            }
            Connector::DirectWithStreams(connection) => {
                log::info!("✅ Using pre-opened Iroh connection with streams");
                Ok(connection)
            }
            Connector::Lazy(addr) => {
                let config = IrohConfig::default();
                let endpoint = create_iroh_endpoint(&config).await
                    .map_err(|e| RSocketError::Other(anyhow::anyhow!("Failed to create endpoint: {}", e).into()))?;
                
                let node_addr = parse_node_addr(&addr)?;
                
                log::info!("🔗 Connecting to NodeAddr: {:?}", node_addr);
                log::info!("   - NodeId: {}", node_addr.node_id);
                log::info!("   - Relay: {:?}", node_addr.relay_url);
                log::info!("   - Direct addresses: {:?}", node_addr.direct_addresses);
                
                let connection = endpoint.connect(node_addr, RSOCKET_ALPN)
                    .await
                    .map_err(|e| RSocketError::Other(anyhow::anyhow!("Failed to connect: {}", e).into()))?;
                
                log::info!("🔗 Opening bidirectional stream for RSocket communication");
                let (send_stream, recv_stream) = connection.open_bi()
                    .await
                    .map_err(|e| RSocketError::Other(e.into()))?;
                
                log::info!("✅ Bidirectional stream opened successfully");
                Ok(IrohConnectionWithStreams::new(send_stream, recv_stream))
            }
            Connector::NodeAddr(node_addr) => {
                let config = IrohConfig::default();
                let endpoint = create_iroh_endpoint(&config).await
                    .map_err(|e| RSocketError::Other(anyhow::anyhow!("Failed to create endpoint: {}", e).into()))?;
                
                log::info!("🔗 Connecting to NodeAddr with direct addressing: {:?}", node_addr);
                
                let connection = endpoint.connect(node_addr, RSOCKET_ALPN)
                    .await
                    .map_err(|e| RSocketError::Other(anyhow::anyhow!("Failed to connect to NodeAddr: {}", e).into()))?;
                
                log::info!("🔗 Opening bidirectional stream for RSocket communication");
                let (send_stream, recv_stream) = connection.open_bi()
                    .await
                    .map_err(|e| RSocketError::Other(e.into()))?;
                
                log::info!("✅ Bidirectional stream opened successfully");
                Ok(IrohConnectionWithStreams::new(send_stream, recv_stream))
            }
        }
    }
}

impl From<String> for IrohClientTransport {
    fn from(addr: String) -> Self {
        IrohClientTransport {
            connector: Connector::Lazy(addr),
        }
    }
}

impl From<&str> for IrohClientTransport {
    fn from(addr: &str) -> Self {
        IrohClientTransport {
            connector: Connector::Lazy(addr.to_string()),
        }
    }
}

impl IrohClientTransport {
    pub fn from_connection(connection: iroh::endpoint::Connection) -> Self {
        IrohClientTransport {
            connector: Connector::Direct(connection),
        }
    }
    
    pub fn from_node_addr(node_addr: iroh::NodeAddr) -> Self {
        IrohClientTransport {
            connector: Connector::NodeAddr(node_addr),
        }
    }

    pub fn from_connection_with_streams(connection: IrohConnectionWithStreams) -> Self {
        IrohClientTransport {
            connector: Connector::DirectWithStreams(connection),
        }
    }
}

impl From<iroh::endpoint::Connection> for IrohClientTransport {
    fn from(connection: iroh::endpoint::Connection) -> Self {
        IrohClientTransport {
            connector: Connector::Direct(connection),
        }
    }
}
