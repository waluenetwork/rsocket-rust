use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use libc::size_t;

#[repr(C)]
pub struct GoRSocketPerformanceMetrics {
    request_count: Arc<AtomicU64>,
    response_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    bytes_received: Arc<AtomicU64>,
    start_time: u64,
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_create() -> *mut GoRSocketPerformanceMetrics {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    Box::into_raw(Box::new(GoRSocketPerformanceMetrics {
        request_count: Arc::new(AtomicU64::new(0)),
        response_count: Arc::new(AtomicU64::new(0)),
        error_count: Arc::new(AtomicU64::new(0)),
        bytes_sent: Arc::new(AtomicU64::new(0)),
        bytes_received: Arc::new(AtomicU64::new(0)),
        start_time,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_record_request(
    metrics: *mut GoRSocketPerformanceMetrics,
    bytes_sent: size_t,
) {
    if !metrics.is_null() {
        unsafe {
            (*metrics).request_count.fetch_add(1, Ordering::Relaxed);
            (*metrics).bytes_sent.fetch_add(bytes_sent as u64, Ordering::Relaxed);
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_record_response(
    metrics: *mut GoRSocketPerformanceMetrics,
    bytes_received: size_t,
) {
    if !metrics.is_null() {
        unsafe {
            (*metrics).response_count.fetch_add(1, Ordering::Relaxed);
            (*metrics).bytes_received.fetch_add(bytes_received as u64, Ordering::Relaxed);
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_record_error(
    metrics: *mut GoRSocketPerformanceMetrics,
) {
    if !metrics.is_null() {
        unsafe {
            (*metrics).error_count.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_get_request_count(
    metrics: *const GoRSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe {
        (*metrics).request_count.load(Ordering::Relaxed)
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_get_response_count(
    metrics: *const GoRSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe {
        (*metrics).response_count.load(Ordering::Relaxed)
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_get_error_count(
    metrics: *const GoRSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe {
        (*metrics).error_count.load(Ordering::Relaxed)
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_get_bytes_sent(
    metrics: *const GoRSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe {
        (*metrics).bytes_sent.load(Ordering::Relaxed)
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_get_bytes_received(
    metrics: *const GoRSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    unsafe {
        (*metrics).bytes_received.load(Ordering::Relaxed)
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_get_uptime_seconds(
    metrics: *const GoRSocketPerformanceMetrics,
) -> u64 {
    if metrics.is_null() {
        return 0;
    }
    
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    unsafe {
        current_time - (*metrics).start_time
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_performance_metrics_free(metrics: *mut GoRSocketPerformanceMetrics) {
    if !metrics.is_null() {
        unsafe {
            let _ = Box::from_raw(metrics);
        }
    }
}
