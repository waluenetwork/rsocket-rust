use iroh::{Endpoint, NodeAddr, NodeId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use rsocket_rust::error::RSocketError;

pub const RSOCKET_ALPN: &[u8] = b"rsocket-iroh";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrohConfig {
    pub node_id: Option<NodeId>,
    pub relay_url: Option<String>,
    pub bind_port: Option<u16>,
}

impl Default for IrohConfig {
    fn default() -> Self {
        Self {
            node_id: None,
            relay_url: None,
            bind_port: None,
        }
    }
}

pub async fn create_iroh_endpoint(config: &IrohConfig) -> std::result::Result<Endpoint, Box<dyn std::error::Error>> {
    let mut builder = Endpoint::builder();
    
    builder = builder.discovery_n0();
    
    builder = builder.alpns(vec![RSOCKET_ALPN.to_vec()]);
    
    if let Some(port) = config.bind_port {
        let socket_addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::UNSPECIFIED, port);
        builder = builder.bind_addr_v4(socket_addr);
    }
    
    let endpoint = builder.bind().await?;
    
    log::info!("Iroh endpoint created with NodeId: {}", endpoint.node_id());
    
    Ok(endpoint)
}

pub fn parse_node_addr(addr: &str) -> rsocket_rust::Result<NodeAddr> {
    if let Ok(node_id) = NodeId::from_str(addr) {
        return Ok(NodeAddr::from_parts(node_id, None, vec![]));
    }
    
    Err(RSocketError::Other(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("Invalid NodeAddr format: {}. Expected NodeId format.", addr)
    ).into()).into())
}

pub fn create_local_node_addr(node_id: NodeId, direct_addresses: Vec<std::net::SocketAddr>) -> NodeAddr {
    NodeAddr {
        node_id,
        relay_url: None,
        direct_addresses: direct_addresses.into_iter().collect(),
    }
}
