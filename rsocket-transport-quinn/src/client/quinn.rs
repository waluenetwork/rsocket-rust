use std::net::SocketAddr;

use quinn::{ClientConfig, Endpoint, Connection};
use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::Transport, Result};

use crate::{connection::QuinnConnection, misc::create_client_config};

#[derive(Debug)]
enum Connector {
    Direct(QuinnConnection),
    Lazy(String, Option<ClientConfig>),
}

#[derive(Debug)]
pub struct QuinnClientTransport {
    connector: Connector,
}

#[async_trait]
impl Transport for QuinnClientTransport {
    type Conn = QuinnConnection;

    async fn connect(self) -> Result<QuinnConnection> {
        match self.connector {
            Connector::Direct(quinn_connection) => {
                Ok(quinn_connection)
            }
            Connector::Lazy(addr, config) => {
                let config = config.unwrap_or_else(|| create_client_config());
                let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
                    .map_err(|e| RSocketError::Other(e.into()))?;
                endpoint.set_default_client_config(config);
                
                let (host, port) = parse_address(&addr)?;
                let socket_addr: SocketAddr = format!("{}:{}", host, port).parse()
                    .map_err(|e: std::net::AddrParseError| RSocketError::Other(e.into()))?;
                
                let connection = endpoint.connect(socket_addr, &host)
                    .map_err(|e| RSocketError::Other(e.into()))?
                    .await
                    .map_err(|e| RSocketError::Other(e.into()))?;
                
                let (send_stream, recv_stream) = connection.open_bi()
                    .await
                    .map_err(|e| RSocketError::Other(e.into()))?;
                
                Ok(QuinnConnection::new(send_stream, recv_stream))
            }
        }
    }
}

impl From<String> for QuinnClientTransport {
    fn from(addr: String) -> Self {
        QuinnClientTransport {
            connector: Connector::Lazy(addr, None),
        }
    }
}

impl From<&str> for QuinnClientTransport {
    fn from(addr: &str) -> Self {
        QuinnClientTransport {
            connector: Connector::Lazy(addr.to_string(), None),
        }
    }
}

impl QuinnClientTransport {
    pub fn from_connection(_connection: Connection) -> Self {
        panic!("from_connection should not be used - use from_quinn_connection instead");
    }
    
    pub fn from_quinn_connection(quinn_connection: QuinnConnection) -> Self {
        QuinnClientTransport {
            connector: Connector::Direct(quinn_connection),
        }
    }
}

impl From<QuinnConnection> for QuinnClientTransport {
    fn from(quinn_connection: QuinnConnection) -> Self {
        QuinnClientTransport {
            connector: Connector::Direct(quinn_connection),
        }
    }
}

fn parse_address(addr: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 2 {
        return Err(RSocketError::Other(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid address format, expected host:port"
        ).into()).into());
    }
    
    let host = parts[0].to_string();
    let port = parts[1].parse::<u16>()
        .map_err(|e| RSocketError::Other(e.into()))?;
    
    Ok((host, port))
}
