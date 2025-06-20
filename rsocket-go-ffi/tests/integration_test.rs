#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_go_init() {
        let result = rsocket_go_init();
        assert_eq!(result, 0);
    }
    
    #[test]
    fn test_go_version() {
        let version = rsocket_go_get_version();
        assert!(!version.is_null());
        
        unsafe {
            let version_str = std::ffi::CStr::from_ptr(version);
            assert!(!version_str.to_string_lossy().is_empty());
            rsocket_go_free_string(version as *mut i8);
        }
    }
    
    #[test]
    fn test_payload_creation() {
        let data = CString::new("test data").unwrap();
        let metadata = CString::new("test metadata").unwrap();
        
        let payload = rsocket_go_payload_create_from_string(
            data.as_ptr(),
            metadata.as_ptr(),
        );
        
        assert!(!payload.is_null());
        
        unsafe {
            let data_len = rsocket_go_payload_get_data_length(payload);
            assert_eq!(data_len, 9); // "test data".len()
            
            let metadata_len = rsocket_go_payload_get_metadata_length(payload);
            assert_eq!(metadata_len, 13); // "test metadata".len()
            
            rsocket_go_payload_free(payload);
        }
    }
    
    #[test]
    fn test_client_creation() {
        let client = rsocket_go_client_create();
        assert!(!client.is_null());
        
        unsafe {
            rsocket_go_client_free(client);
        }
    }
    
    #[test]
    fn test_performance_metrics() {
        let metrics = rsocket_go_performance_metrics_create();
        assert!(!metrics.is_null());
        
        unsafe {
            rsocket_go_performance_metrics_record_request(metrics, 100);
            rsocket_go_performance_metrics_record_response(metrics, 200);
            rsocket_go_performance_metrics_record_error(metrics);
            
            assert_eq!(rsocket_go_performance_metrics_get_request_count(metrics), 1);
            assert_eq!(rsocket_go_performance_metrics_get_response_count(metrics), 1);
            assert_eq!(rsocket_go_performance_metrics_get_error_count(metrics), 1);
            assert_eq!(rsocket_go_performance_metrics_get_bytes_sent(metrics), 100);
            assert_eq!(rsocket_go_performance_metrics_get_bytes_received(metrics), 200);
            
            rsocket_go_performance_metrics_free(metrics);
        }
    }
    
    #[test]
    fn test_transport_validation() {
        let tcp_addr = CString::new("127.0.0.1:7878").unwrap();
        let result = rsocket_go_validate_tcp_address(tcp_addr.as_ptr());
        assert_eq!(result, 1);
        
        let invalid_addr = CString::new("invalid").unwrap();
        let result = rsocket_go_validate_tcp_address(invalid_addr.as_ptr());
        assert_eq!(result, 0);
        
        let ws_url = CString::new("ws://localhost:8080").unwrap();
        let result = rsocket_go_validate_websocket_url(ws_url.as_ptr());
        assert_eq!(result, 1);
        
        let invalid_url = CString::new("http://localhost").unwrap();
        let result = rsocket_go_validate_websocket_url(invalid_url.as_ptr());
        assert_eq!(result, 0);
    }
    
    #[test]
    fn test_supported_transports() {
        let transports = rsocket_go_get_supported_transports();
        assert!(!transports.is_null());
        
        unsafe {
            let transports_str = std::ffi::CStr::from_ptr(transports);
            let transports_string = transports_str.to_string_lossy();
            assert!(transports_string.contains("tcp"));
            assert!(transports_string.contains("websocket"));
            rsocket_go_free_string(transports as *mut i8);
        }
    }
}
