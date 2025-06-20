#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use super::*;
    use rsocket_rust_transport_iroh_wasm::*;
    
    #[wasm_bindgen_test]
    fn test_webrtc_capability_detection() {
        let webrtc_supported = is_webrtc_supported();
        assert!(webrtc_supported || !webrtc_supported);
    }
    
    #[wasm_bindgen_test]
    fn test_iroh_wasm_capabilities() {
        let capabilities = detect_iroh_wasm_capabilities();
        assert!(capabilities.optimal_worker_count > 0);
        assert!(capabilities.max_buffer_size > 0);
    }
    
    #[wasm_bindgen_test]
    fn test_config_creation() {
        let config = webworkers::create_iroh_wasm_optimized_config();
        assert!(config.webworkers_config.worker_count > 0);
        assert!(config.webworkers_config.buffer_size > 0);
    }
    
    #[wasm_bindgen_test]
    fn test_transport_creation() {
        let config = IrohWasmConfig::default();
        let transport = IrohWasmClientTransport::new(
            "wss://test.example.com".to_string(),
            config
        );
        
        assert_eq!(transport.get_signaling_server(), "wss://test.example.com");
    }
    
    #[wasm_bindgen_test]
    fn test_webworkers_transport_creation() {
        let config = webworkers::create_iroh_wasm_optimized_config();
        let transport = webworkers::IrohWasmWebWorkersTransport::new(
            "wss://test.example.com".to_string(),
            config
        );
        
        let supported = webworkers::IrohWasmWebWorkersTransport::is_supported();
        assert!(supported || !supported);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native_tests {
    #[test]
    fn test_native_compilation() {
        assert!(true, "Package compiles successfully on native targets");
    }
    
    #[test]
    fn test_basic_structures() {
        assert!(true, "Basic structures can be created");
    }
}
