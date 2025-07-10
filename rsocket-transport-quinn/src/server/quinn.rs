use std::net::SocketAddr;

use quinn::Endpoint;
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::ServerTransport, Result};

use crate::{client::QuinnClientTransport, connection::QuinnConnection, misc::create_server_config};

#[derive(Debug)]
pub struct QuinnServerTransport {
    addr: SocketAddr,
    endpoint: Option<Endpoint>,
}

impl QuinnServerTransport {
    fn new(addr: SocketAddr) -> QuinnServerTransport {
        QuinnServerTransport {
            addr,
            endpoint: None,
        }
    }
}

#[async_trait]
impl ServerTransport for QuinnServerTransport {
    type Item = QuinnClientTransport;

    async fn start(&mut self) -> Result<()> {
        if self.endpoint.is_some() {
            return Ok(());
        }
        
        let config = create_server_config();
        let endpoint = Endpoint::server(config, self.addr)
            .map_err(|e| RSocketError::Other(e.into()))?;
        
        self.endpoint = Some(endpoint);
        log::debug!("QUIC server listening on: {}", &self.addr);
        Ok(())
    }

    async fn next(&mut self) -> Option<Result<Self::Item>> {
        match self.endpoint.as_mut() {
            Some(endpoint) => {
                match endpoint.accept().await {
                    Some(connecting) => {
                        match connecting.await {
                            Ok(connection) => {
                                match connection.accept_bi().await {
                                    Ok((send_stream, recv_stream)) => {
                                        let quinn_connection = QuinnConnection::new(send_stream, recv_stream);
                                        Some(Ok(QuinnClientTransport::from_quinn_connection(quinn_connection)))
                                    }
                                    Err(e) => Some(Err(RSocketError::Other(e.into()).into())),
                                }
                            }
                            Err(e) => Some(Err(RSocketError::Other(e.into()).into())),
                        }
                    }
                    None => None,
                }
            }
            None => None,
        }
    }
}

impl From<SocketAddr> for QuinnServerTransport {
    fn from(addr: SocketAddr) -> QuinnServerTransport {
        QuinnServerTransport::new(addr)
    }
}

impl From<String> for QuinnServerTransport {
    fn from(addr: String) -> QuinnServerTransport {
        let socket_addr: SocketAddr = addr.parse().expect("Invalid address format");
        QuinnServerTransport::new(socket_addr)
    }
}

impl From<&str> for QuinnServerTransport {
    fn from(addr: &str) -> QuinnServerTransport {
        let socket_addr: SocketAddr = addr.parse().expect("Invalid address format");
        QuinnServerTransport::new(socket_addr)
    }
}
