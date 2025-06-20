#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{TransportType, TransportCapability, PlatformCapabilities};

    #[test]
    fn test_transport_capabilities() {
        let tcp_caps = TransportType::Tcp.capabilities();
        assert!(tcp_caps.contains(&TransportCapability::NativeNetworking));
        assert!(tcp_caps.contains(&TransportCapability::TlsSupport));

        let websocket_caps = TransportType::WebSocket.capabilities();
        assert!(websocket_caps.contains(&TransportCapability::WebSocket));
        assert!(websocket_caps.contains(&TransportCapability::WasmCompatible));

        let quinn_caps = TransportType::Quinn.capabilities();
        assert!(quinn_caps.contains(&TransportCapability::Quic));
        assert!(quinn_caps.contains(&TransportCapability::NativeNetworking));
    }

    #[test]
    fn test_platform_detection() {
        let platform_caps = PlatformCapabilities::detect();
        
        assert!(!platform_caps.get_preferred_transports().is_empty());

        #[cfg(target_arch = "wasm32")]
        {
            assert!(platform_caps.has_capability(&TransportCapability::WasmCompatible));
            assert!(platform_caps.supports_transport(&TransportType::WebSocket));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            assert!(platform_caps.has_capability(&TransportCapability::NativeNetworking));
            assert!(platform_caps.supports_transport(&TransportType::Tcp));
        }
    }

    #[test]
    fn test_transport_selection() {
        use crate::core::RSocketFactory;

        let builder = RSocketFactory::connect_universal();
        let selected = builder.select_best_transport();
        
        assert!(selected.is_ok());
        
        let transport_type = selected.unwrap();
        let platform_caps = PlatformCapabilities::detect();
        
        assert!(platform_caps.supports_transport(&transport_type));
    }

    #[test]
    fn test_preferred_transport() {
        use crate::core::RSocketFactory;

        let builder = RSocketFactory::connect_universal()
            .prefer_transport(TransportType::WebSocket);
        
        let platform_caps = PlatformCapabilities::detect();
        
        if platform_caps.supports_transport(&TransportType::WebSocket) {
            let selected = builder.select_best_transport().unwrap();
            assert_eq!(selected, TransportType::WebSocket);
        }
    }

    #[test]
    fn test_fallback_disabled() {
        use crate::core::RSocketFactory;

        let builder = RSocketFactory::connect_universal()
            .prefer_transport(TransportType::IrohP2P)
            .disable_fallback();
        
        let platform_caps = PlatformCapabilities::detect();
        
        if !platform_caps.supports_transport(&TransportType::IrohP2P) {
            let result = builder.select_best_transport();
            assert!(result.is_err());
        }
    }
}
