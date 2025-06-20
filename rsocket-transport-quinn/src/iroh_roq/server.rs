use std::net::{SocketAddr, SocketAddrV4};
use iroh::Endpoint;
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::ServerTransport, Result};
use anyhow;

use super::client::IrohRoqClientTransport;
use super::session::IrohRoqSessionConfig;

#[derive(Debug)]
pub struct IrohRoqServerTransport {
    endpoint: Option<Endpoint>,
    bind_addr: SocketAddr,
    config: IrohRoqSessionConfig,
}

impl IrohRoqServerTransport {
    pub fn new(bind_addr: SocketAddrV4, config: IrohRoqSessionConfig) -> Self {
        Self {
            endpoint: None,
            bind_addr: bind_addr.into(),
            config,
        }
    }

    pub fn with_defaults(bind_addr: SocketAddrV4) -> Self {
        let config = IrohRoqSessionConfig::default();
        Self::new(bind_addr, config)
    }
}

#[async_trait]
impl ServerTransport for IrohRoqServerTransport {
    type Item = IrohRoqClientTransport;

    async fn start(&mut self) -> Result<()> {
        if self.endpoint.is_some() {
            return Ok(());
        }

        let bind_addr_v4: SocketAddrV4 = match self.bind_addr {
            SocketAddr::V4(addr) => addr,
            SocketAddr::V6(_) => return Err(RSocketError::Other(anyhow::anyhow!("IPv6 not supported for iroh-roq")).into()),
        };
        
        let endpoint = Endpoint::builder()
            .bind_addr_v4(bind_addr_v4)
            .alpns(vec![b"/iroh/roq/1".to_vec()])
            .bind()
            .await
            .map_err(|e| RSocketError::Other(e.into()))?;

        self.endpoint = Some(endpoint);
        log::debug!("iroh-roq server listening on: {}", self.bind_addr);
        Ok(())
    }

    async fn next(&mut self) -> Option<Result<Self::Item>> {
        match self.endpoint.as_mut() {
            Some(endpoint) => {
                match endpoint.accept().await {
                    Some(connecting) => {
                        match connecting.await {
                            Ok(_connection) => {
                                match IrohRoqClientTransport::with_defaults(self.bind_addr).await {
                                    Ok(transport) => Some(Ok(transport)),
                                    Err(e) => Some(Err(e)),
                                }
                            }
                            Err(e) => Some(Err(RSocketError::Other(anyhow::anyhow!("Connection error: {}", e)).into())),
                        }
                    }
                    None => None,
                }
            }
            None => None,
        }
    }
}
