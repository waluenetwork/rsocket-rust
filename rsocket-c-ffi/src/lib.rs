use std::ffi::CString;
use std::os::raw::{c_char, c_int};

mod client;
mod server;
mod transport;
mod payload;
mod performance;

pub use client::*;
pub use server::*;
pub use transport::*;
pub use payload::*;
pub use performance::*;

#[no_mangle]
pub extern "C" fn rsocket_init() -> c_int {
    env_logger::init();
    0
}

#[no_mangle]
pub extern "C" fn rsocket_get_version() -> *const c_char {
    let version = CString::new(env!("CARGO_PKG_VERSION")).unwrap();
    version.into_raw()
}

#[no_mangle]
pub extern "C" fn rsocket_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[repr(C)]
pub struct RSocketError {
    pub code: c_int,
    pub message: *const c_char,
}

#[no_mangle]
pub extern "C" fn rsocket_error_free(error: *mut RSocketError) {
    if !error.is_null() {
        unsafe {
            rsocket_free_string((*error).message as *mut c_char);
            let _ = Box::from_raw(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::RSocketTransportType;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_library_initialization() {
        let result = rsocket_init();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_version_string() {
        let version_ptr = rsocket_get_version();
        assert!(!version_ptr.is_null());
        
        let version_str = unsafe { 
            std::ffi::CStr::from_ptr(version_ptr).to_string_lossy() 
        };
        assert!(!version_str.is_empty());
        
        rsocket_free_string(version_ptr as *mut i8);
    }

    #[test]
    fn test_payload_creation() {
        let data = b"test data";
        let metadata = b"test metadata";
        
        let payload = rsocket_payload_create(
            data.as_ptr(),
            data.len(),
            metadata.as_ptr(),
            metadata.len(),
        );
        
        assert!(!payload.is_null());
        
        let data_len = rsocket_payload_get_data_length(payload);
        assert_eq!(data_len, data.len());
        
        let metadata_len = rsocket_payload_get_metadata_length(payload);
        assert_eq!(metadata_len, metadata.len());
        
        rsocket_payload_free(payload);
    }

    #[test]
    fn test_payload_from_string() {
        let data_str = CString::new("Hello, World!").unwrap();
        let metadata_str = CString::new("metadata").unwrap();
        
        let payload = rsocket_payload_create_from_string(
            data_str.as_ptr(),
            metadata_str.as_ptr(),
        );
        
        assert!(!payload.is_null());
        
        let data_len = rsocket_payload_get_data_length(payload);
        assert_eq!(data_len, 13); // "Hello, World!" length
        
        let metadata_len = rsocket_payload_get_metadata_length(payload);
        assert_eq!(metadata_len, 8); // "metadata" length
        
        rsocket_payload_free(payload);
    }

    #[test]
    fn test_client_creation() {
        let client = rsocket_client_create();
        assert!(!client.is_null());
        
        let is_connected = rsocket_client_is_connected(client);
        assert_eq!(is_connected, 0); // Not connected initially
        
        rsocket_client_free(client);
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = rsocket_performance_metrics_create();
        assert!(!metrics.is_null());
        
        let request_count = rsocket_performance_metrics_get_request_count(metrics);
        assert_eq!(request_count, 0);
        
        let response_count = rsocket_performance_metrics_get_response_count(metrics);
        assert_eq!(response_count, 0);
        
        let error_count = rsocket_performance_metrics_get_error_count(metrics);
        assert_eq!(error_count, 0);
        
        rsocket_performance_metrics_record_request(metrics, 100);
        let request_count_after = rsocket_performance_metrics_get_request_count(metrics);
        assert_eq!(request_count_after, 1);
        
        let bytes_sent = rsocket_performance_metrics_get_bytes_sent(metrics);
        assert_eq!(bytes_sent, 100);
        
        rsocket_performance_metrics_record_response(metrics, 200);
        let response_count_after = rsocket_performance_metrics_get_response_count(metrics);
        assert_eq!(response_count_after, 1);
        
        let bytes_received = rsocket_performance_metrics_get_bytes_received(metrics);
        assert_eq!(bytes_received, 200);
        
        rsocket_performance_metrics_record_error(metrics);
        let error_count_after = rsocket_performance_metrics_get_error_count(metrics);
        assert_eq!(error_count_after, 1);
        
        let uptime = rsocket_performance_metrics_get_uptime_seconds(metrics);
        assert!(uptime >= 0);
        
        rsocket_performance_metrics_free(metrics);
    }

    #[test]
    fn test_transport_utilities() {
        let supported_transports = rsocket_get_supported_transports();
        assert!(!supported_transports.is_null());
        
        let transports_str = unsafe {
            std::ffi::CStr::from_ptr(supported_transports).to_string_lossy()
        };
        assert!(transports_str.contains("tcp"));
        assert!(transports_str.contains("websocket"));
        
        rsocket_free_string(supported_transports as *mut i8);
        
        let tcp_str = CString::new("tcp").unwrap();
        let _transport_type = rsocket_parse_transport_type(tcp_str.as_ptr());
        
        let ws_str = CString::new("websocket").unwrap();
        let _transport_type = rsocket_parse_transport_type(ws_str.as_ptr());
        
        let tcp_supported = rsocket_is_transport_supported(RSocketTransportType::TCP);
        assert_eq!(tcp_supported, 1);
        
        let ws_supported = rsocket_is_transport_supported(RSocketTransportType::WEBSOCKET);
        assert_eq!(ws_supported, 1);
        
        let quic_supported = rsocket_is_transport_supported(RSocketTransportType::QUIC);
        assert_eq!(quic_supported, 0); // Not yet supported
    }

    #[test]
    fn test_server_creation() {
        let server = rsocket_server_create();
        assert!(!server.is_null());
        
        rsocket_server_free(server);
    }

    #[test]
    fn test_error_handling() {
        let result = rsocket_payload_get_data_length(ptr::null());
        assert_eq!(result, 0);
        
        let result = rsocket_payload_get_metadata_length(ptr::null());
        assert_eq!(result, 0);
        
        let result = rsocket_client_is_connected(ptr::null());
        assert_eq!(result, 0);
        
        rsocket_payload_free(ptr::null_mut());
        rsocket_client_free(ptr::null_mut());
        rsocket_server_free(ptr::null_mut());
        rsocket_performance_metrics_free(ptr::null_mut());
        rsocket_error_free(ptr::null_mut());
        rsocket_free_string(ptr::null_mut());
    }
}
