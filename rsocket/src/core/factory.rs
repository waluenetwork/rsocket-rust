use super::{Client, ClientBuilder, ServerBuilder};
use crate::transport::{Connection, ServerTransport, Transport, PlatformCapabilities, TransportType};
use crate::error::RSocketError;
use crate::Result;

#[derive(Debug)]
pub struct RSocketFactory;

#[derive(Debug)]
pub struct UniversalClientBuilder {
    platform_caps: PlatformCapabilities,
    preferred_transport: Option<TransportType>,
    fallback_enabled: bool,
}

#[derive(Debug)]
pub struct UniversalServerBuilder {
    platform_caps: PlatformCapabilities,
    transport_types: Vec<TransportType>,
}

impl RSocketFactory {
    pub fn connect<T, C>() -> ClientBuilder<T, C>
    where
        T: Send + Sync + Transport<Conn = C>,
        C: Send + Sync + Connection,
    {
        ClientBuilder::new()
    }

    pub fn receive<S, T>() -> ServerBuilder<S, T>
    where
        S: Send + Sync + ServerTransport<Item = T>,
        T: Send + Sync + Transport,
    {
        ServerBuilder::new()
    }

    pub fn connect_universal() -> UniversalClientBuilder {
        UniversalClientBuilder::new()
    }

    pub fn receive_universal() -> UniversalServerBuilder {
        UniversalServerBuilder::new()
    }
}

impl UniversalClientBuilder {
    pub fn new() -> Self {
        Self {
            platform_caps: PlatformCapabilities::detect(),
            preferred_transport: None,
            fallback_enabled: true,
        }
    }

    pub fn prefer_transport(mut self, transport_type: TransportType) -> Self {
        self.preferred_transport = Some(transport_type);
        self
    }

    pub fn disable_fallback(mut self) -> Self {
        self.fallback_enabled = false;
        self
    }

    pub fn select_best_transport(&self) -> Result<TransportType> {
        if let Some(preferred) = &self.preferred_transport {
            if self.platform_caps.supports_transport(preferred) {
                return Ok(preferred.clone());
            } else if !self.fallback_enabled {
                return Err(RSocketError::TransportNotSupported(format!(
                    "Preferred transport {:?} is not supported on this platform", preferred
                )).into());
            }
        }

        for transport_type in self.platform_caps.get_preferred_transports() {
            if self.platform_caps.supports_transport(transport_type) {
                return Ok(transport_type.clone());
            }
        }

        Err(RSocketError::TransportNotSupported(
            "No supported transport found for this platform".to_string()
        ).into())
    }
}

impl UniversalServerBuilder {
    pub fn new() -> Self {
        Self {
            platform_caps: PlatformCapabilities::detect(),
            transport_types: Vec::new(),
        }
    }

    pub fn add_transport(mut self, transport_type: TransportType) -> Result<Self> {
        if self.platform_caps.supports_transport(&transport_type) {
            self.transport_types.push(transport_type);
            Ok(self)
        } else {
            Err(RSocketError::TransportNotSupported(format!(
                "Transport {:?} is not supported on this platform", transport_type
            )).into())
        }
    }

    pub fn add_all_supported_transports(mut self) -> Self {
        for transport_type in self.platform_caps.get_preferred_transports() {
            if self.platform_caps.supports_transport(transport_type) {
                self.transport_types.push(transport_type.clone());
            }
        }
        self
    }
}
