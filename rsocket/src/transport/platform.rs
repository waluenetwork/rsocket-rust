use std::collections::HashSet;
use super::capability::{TransportCapability, TransportType};

#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    available_capabilities: HashSet<TransportCapability>,
    preferred_transports: Vec<TransportType>,
}

impl PlatformCapabilities {
    pub fn detect() -> Self {
        let mut capabilities = HashSet::new();
        let mut preferred_transports = Vec::new();

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                capabilities.insert(TransportCapability::WasmCompatible);
                capabilities.insert(TransportCapability::WebSocket);
                
                if Self::has_webtransport_support() {
                    capabilities.insert(TransportCapability::WebTransport);
                    capabilities.insert(TransportCapability::Quic);
                    preferred_transports.push(TransportType::WebTransport);
                }
                
                preferred_transports.push(TransportType::WebSocket);
                preferred_transports.push(TransportType::Wasm);
            } else if #[cfg(target_family = "unix")] {
                capabilities.insert(TransportCapability::NativeNetworking);
                capabilities.insert(TransportCapability::TlsSupport);
                capabilities.insert(TransportCapability::UnixSockets);
                capabilities.insert(TransportCapability::Quic);
                capabilities.insert(TransportCapability::IrohP2P);
                
                preferred_transports.push(TransportType::IrohP2P);
                preferred_transports.push(TransportType::Quinn);
                preferred_transports.push(TransportType::Tcp);
                preferred_transports.push(TransportType::Unix);
            } else if #[cfg(target_family = "windows")] {
                capabilities.insert(TransportCapability::NativeNetworking);
                capabilities.insert(TransportCapability::TlsSupport);
                capabilities.insert(TransportCapability::Quic);
                capabilities.insert(TransportCapability::IrohP2P);
                
                preferred_transports.push(TransportType::IrohP2P);
                preferred_transports.push(TransportType::Quinn);
                preferred_transports.push(TransportType::Tcp);
            } else {
                capabilities.insert(TransportCapability::NativeNetworking);
                preferred_transports.push(TransportType::Tcp);
            }
        }

        if !cfg!(target_arch = "wasm32") {
            capabilities.insert(TransportCapability::WebSocket);
            preferred_transports.push(TransportType::WebSocket);
        }

        Self {
            available_capabilities: capabilities,
            preferred_transports,
        }
    }

    pub fn has_capability(&self, capability: &TransportCapability) -> bool {
        self.available_capabilities.contains(capability)
    }

    pub fn supports_transport(&self, transport_type: &TransportType) -> bool {
        let required_caps = transport_type.capabilities();
        required_caps.iter().all(|cap| self.has_capability(cap))
    }

    pub fn get_preferred_transports(&self) -> &[TransportType] {
        &self.preferred_transports
    }

    #[cfg(target_arch = "wasm32")]
    fn has_webtransport_support() -> bool {
        true
    }
}
