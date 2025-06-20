use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransportCapability {
    NativeNetworking,
    WebSocket,
    Quic,
    IrohP2P,
    WebTransport,
    WasmCompatible,
    TlsSupport,
    UnixSockets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportType {
    Tcp,
    WebSocket,
    Quinn,
    IrohP2P,
    WebTransport,
    Wasm,
    Tls,
    Unix,
}

impl TransportType {
    pub fn capabilities(&self) -> HashSet<TransportCapability> {
        let mut caps = HashSet::new();
        match self {
            TransportType::Tcp => {
                caps.insert(TransportCapability::NativeNetworking);
                caps.insert(TransportCapability::TlsSupport);
            }
            TransportType::WebSocket => {
                caps.insert(TransportCapability::WebSocket);
                caps.insert(TransportCapability::WasmCompatible);
            }
            TransportType::Quinn => {
                caps.insert(TransportCapability::NativeNetworking);
                caps.insert(TransportCapability::Quic);
                caps.insert(TransportCapability::TlsSupport);
            }
            TransportType::IrohP2P => {
                caps.insert(TransportCapability::IrohP2P);
                caps.insert(TransportCapability::NativeNetworking);
            }
            TransportType::WebTransport => {
                caps.insert(TransportCapability::WebTransport);
                caps.insert(TransportCapability::Quic);
                caps.insert(TransportCapability::WasmCompatible);
            }
            TransportType::Wasm => {
                caps.insert(TransportCapability::WasmCompatible);
                caps.insert(TransportCapability::WebSocket);
            }
            TransportType::Tls => {
                caps.insert(TransportCapability::NativeNetworking);
                caps.insert(TransportCapability::TlsSupport);
            }
            TransportType::Unix => {
                caps.insert(TransportCapability::NativeNetworking);
                caps.insert(TransportCapability::UnixSockets);
            }
        }
        caps
    }
}
