
use std::time::{SystemTime, UNIX_EPOCH};
use libc::size_t;

#[repr(C)]
pub struct RSocketPerformanceMetrics {
    request_count: u64,
    response_count: u64,
    error_count: u64,
    bytes_sent: u64,
    bytes_received: u64,
    start_time: u64,
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_create() -> *mut RSocketPerformanceMetrics {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Box::into_raw(Box::new(RSocketPerformanceMetrics {
        request_count: 0,
        response_count: 0,
        error_count: 0,
        bytes_sent: 0,
        bytes_received: 0,
        start_time,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_record_request(
    metrics: *mut RSocketPerformanceMetrics,
    bytes_sent: size_t,
) {
    if !metrics.is_null() {
        unsafe {
            (*metrics).request_count += 1;
            (*metrics).bytes_sent += bytes_sent as u64;
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_record_response(
    metrics: *mut RSocketPerformanceMetrics,
    bytes_received: size_t,
) {
    if !metrics.is_null() {
        unsafe {
            (*metrics).response_count += 1;
            (*metrics).bytes_received += bytes_received as u64;
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_record_error(metrics: *mut RSocketPerformanceMetrics) {
    if !metrics.is_null() {
        unsafe {
            (*metrics).error_count += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_get_request_count(
    metrics: *const RSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe { (*metrics).request_count }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_get_response_count(
    metrics: *const RSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe { (*metrics).response_count }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_get_error_count(
    metrics: *const RSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe { (*metrics).error_count }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_get_bytes_sent(
    metrics: *const RSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe { (*metrics).bytes_sent }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_get_bytes_received(
    metrics: *const RSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe { (*metrics).bytes_received }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_get_uptime_seconds(
    metrics: *const RSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    unsafe { current_time - (*metrics).start_time }
}

#[no_mangle]
pub extern "C" fn rsocket_performance_metrics_free(metrics: *mut RSocketPerformanceMetrics) {
    if !metrics.is_null() {
        unsafe {
            let _ = Box::from_raw(metrics);
        }
    }
}
