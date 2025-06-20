use std::net::{SocketAddr, SocketAddrV4};
use iroh::{Endpoint, NodeAddr};
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::Transport, Result};

use super::connection::IrohRoqConnection;
use super::session::IrohRoqSessionConfig;

#[derive(Debug)]
pub struct IrohRoqClientTransport {
    endpoint: Endpoint,
    server_addr: SocketAddr,
    config: IrohRoqSessionConfig,
}

impl IrohRoqClientTransport {
    pub async fn new(
        bind_addr: SocketAddrV4,
        server_addr: SocketAddr,
        config: IrohRoqSessionConfig,
    ) -> Result<Self> {
        let endpoint = Endpoint::builder()
            .bind_addr_v4(bind_addr)
            .alpns(vec![b"/iroh/roq/1".to_vec()])
            .bind()
            .await
            .map_err(|e| RSocketError::Other(e.into()))?;

        Ok(Self {
            endpoint,
            server_addr,
            config,
        })
    }

    pub async fn with_defaults(server_addr: SocketAddr) -> Result<Self> {
        let bind_addr: SocketAddrV4 = "0.0.0.0:0".parse().unwrap();
        let config = IrohRoqSessionConfig::default();
        Self::new(bind_addr, server_addr, config).await
    }
}

#[async_trait]
impl Transport for IrohRoqClientTransport {
    type Conn = IrohRoqConnection;

    async fn connect(self) -> Result<IrohRoqConnection> {
        let node_addr = NodeAddr::from((
            iroh::PublicKey::from_bytes(&[0u8; 32]).unwrap(), // Placeholder key
            None,
            &[self.server_addr][..],
        ));
        
        let conn = self.endpoint
            .connect(node_addr, b"/iroh/roq/1")
            .await
            .map_err(|e| RSocketError::Other(e.into()))?;

        let iroh_conn = IrohRoqConnection::from_quic_connection(conn);
        Ok(iroh_conn)
    }
}

impl From<SocketAddr> for IrohRoqClientTransport {
    fn from(addr: SocketAddr) -> Self {
        let bind_addr: SocketAddrV4 = "0.0.0.0:0".parse().unwrap();
        let config = IrohRoqSessionConfig::default();
        
        let endpoint = tokio::runtime::Handle::try_current()
            .and_then(|handle| {
                let endpoint_builder = Endpoint::builder()
                    .bind_addr_v4(bind_addr)
                    .alpns(vec![b"/iroh/roq/1".to_vec()]);
                Ok(handle.block_on(endpoint_builder.bind()))
            })
            .unwrap_or_else(|_| {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let endpoint_builder = Endpoint::builder()
                    .bind_addr_v4(bind_addr)
                    .alpns(vec![b"/iroh/roq/1".to_vec()]);
                rt.block_on(endpoint_builder.bind())
            })
            .expect("Failed to bind endpoint");
        
        Self {
            endpoint,
            server_addr: addr,
            config,
        }
    }
}
