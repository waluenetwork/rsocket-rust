use quinn::{ClientConfig, ServerConfig, Endpoint};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Once;
use rsocket_rust::error::RSocketError;

pub const RSOCKET_ALPN: &[u8] = b"rsocket-p2p";

static INIT: Once = Once::new();

fn ensure_crypto_provider() {
    INIT.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: SocketAddr,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub peer_id: String,
    pub listen_addr: SocketAddr,
    pub known_peers: HashMap<String, SocketAddr>,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            peer_id: generate_peer_id(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            known_peers: HashMap::new(),
        }
    }
}

pub fn generate_peer_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("peer_{}", timestamp)
}

pub fn create_client_config() -> ClientConfig {
    ensure_crypto_provider();
    
    let mut crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();
    
    crypto.alpn_protocols = vec![RSOCKET_ALPN.to_vec()];

    quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).unwrap()
    ))
}

pub fn create_server_config() -> ServerConfig {
    ensure_crypto_provider();
    
    let cert = generate_self_signed_cert();
    let key = cert.serialize_private_key_der();
    let cert_der = CertificateDer::from(cert.serialize_der().unwrap());
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key));

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .unwrap();
    
    server_crypto.alpn_protocols = vec![RSOCKET_ALPN.to_vec()];

    let server_config = quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto).unwrap();
    ServerConfig::with_crypto(Arc::new(server_config))
}

pub async fn create_p2p_endpoint(config: &P2PConfig) -> std::result::Result<Endpoint, Box<dyn std::error::Error>> {
    let server_config = create_server_config();
    let mut endpoint = Endpoint::server(server_config, config.listen_addr)?;
    
    let client_config = create_client_config();
    endpoint.set_default_client_config(client_config);
    
    Ok(endpoint)
}

fn generate_self_signed_cert() -> rcgen::Certificate {
    let mut params = rcgen::CertificateParams::new(vec!["localhost".to_string()]);
    params.distinguished_name = rcgen::DistinguishedName::new();
    rcgen::Certificate::from_params(params).unwrap()
}

#[derive(Debug)]
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

pub fn parse_peer_address(addr: &str) -> rsocket_rust::Result<(String, SocketAddr)> {
    if let Some((peer_id, socket_addr_str)) = addr.split_once('@') {
        let socket_addr: SocketAddr = socket_addr_str.parse()
            .map_err(|e: std::net::AddrParseError| RSocketError::Other(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid socket address: {}", e)
            ).into()))?;
        Ok((peer_id.to_string(), socket_addr))
    } else {
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e: std::net::AddrParseError| RSocketError::Other(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid socket address: {}", e)
            ).into()))?;
        Ok((generate_peer_id(), socket_addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_peer_address() {
        let addr = "peer_123@127.0.0.1:8080";
        let result = parse_peer_address(addr);
        assert!(result.is_ok());
        let (peer_id, socket_addr) = result.unwrap();
        assert_eq!(peer_id, "peer_123");
        assert_eq!(socket_addr.to_string(), "127.0.0.1:8080");
    }
    
    #[test]
    fn test_parse_peer_address_no_id() {
        let addr = "127.0.0.1:8080";
        let result = parse_peer_address(addr);
        assert!(result.is_ok());
        let (_peer_id, socket_addr) = result.unwrap();
        assert_eq!(socket_addr.to_string(), "127.0.0.1:8080");
    }
}
