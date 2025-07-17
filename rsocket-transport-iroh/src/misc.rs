use iroh::{Endpoint, NodeAddr, NodeId, SecretKey};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use rsocket_rust::error::RSocketError;

pub const RSOCKET_ALPN: &[u8] = b"rsocket-iroh";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrohConfig {
    pub node_id: Option<NodeId>,
    pub relay_url: Option<String>,
    pub bind_port: Option<u16>,
    pub private_key: Option<String>,
}

impl Default for IrohConfig {
    fn default() -> Self {
        Self {
            node_id: None,
            relay_url: None,
            bind_port: None,
            private_key: None,
        }
    }
}

pub async fn create_iroh_endpoint(config: &IrohConfig) -> std::result::Result<Endpoint, Box<dyn std::error::Error>> {
    let mut builder = Endpoint::builder();
    
    if let Some(private_key_str) = &config.private_key {
        let private_key_bytes = hex::decode(private_key_str)
            .map_err(|e| format!("Invalid private key hex: {}", e))?;
        
        if private_key_bytes.len() != 32 {
            return Err(format!("Private key must be exactly 32 bytes (64 hex characters), got {} bytes", private_key_bytes.len()).into());
        }
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&private_key_bytes);
        let secret_key = SecretKey::from_bytes(&key_array);
        builder = builder.secret_key(secret_key);
        log::info!("Using provided private key for Iroh endpoint");
    } else {
        log::info!("No private key provided, Iroh will generate one automatically");
    }
    
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
    if let Ok(node_ticket) = NodeTicket::from_str(addr) {
        let node_addr: NodeAddr = node_ticket.into();
        log::info!("Parsed complete NodeAddr - NodeId: {}, Relay: {:?}, Direct addresses: {:?}", 
                  node_addr.node_id, node_addr.relay_url, node_addr.direct_addresses);
        return Ok(node_addr);
    }
    
    if let Ok(node_id) = NodeId::from_str(addr) {
        log::warn!("Parsed NodeId without relay information. Distributed connections may fail.");
        return Ok(NodeAddr::from_parts(node_id, None, vec![]));
    }
    
    Err(RSocketError::Other(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("Invalid NodeAddr format: {}. Expected NodeTicket or NodeId format.", addr)
    ).into()).into())
}

pub fn create_local_node_addr(node_id: NodeId, direct_addresses: Vec<std::net::SocketAddr>) -> NodeAddr {
    NodeAddr {
        node_id,
        relay_url: None,
        direct_addresses: direct_addresses.into_iter().collect(),
    }
}
