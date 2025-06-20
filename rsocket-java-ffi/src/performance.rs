use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jlong, jint};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};

#[repr(C)]
pub struct JavaRSocketPerformanceMetrics {
    request_count: AtomicU64,
    response_count: AtomicU64,
    error_count: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    start_time: SystemTime,
}

impl JavaRSocketPerformanceMetrics {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            response_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            start_time: SystemTime::now(),
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_create(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let metrics = Box::new(JavaRSocketPerformanceMetrics::new());
    Box::into_raw(metrics) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_recordRequest(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
    bytes: jlong,
) {
    if metrics_ptr == 0 {
        return;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.request_count.fetch_add(1, Ordering::Relaxed);
    metrics.bytes_sent.fetch_add(bytes as u64, Ordering::Relaxed);
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_recordResponse(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
    bytes: jlong,
) {
    if metrics_ptr == 0 {
        return;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.response_count.fetch_add(1, Ordering::Relaxed);
    metrics.bytes_received.fetch_add(bytes as u64, Ordering::Relaxed);
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_recordError(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) {
    if metrics_ptr == 0 {
        return;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.error_count.fetch_add(1, Ordering::Relaxed);
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_getRequestCount(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) -> jlong {
    if metrics_ptr == 0 {
        return 0;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.request_count.load(Ordering::Relaxed) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_getResponseCount(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) -> jlong {
    if metrics_ptr == 0 {
        return 0;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.response_count.load(Ordering::Relaxed) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_getErrorCount(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) -> jlong {
    if metrics_ptr == 0 {
        return 0;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.error_count.load(Ordering::Relaxed) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_getBytesSent(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) -> jlong {
    if metrics_ptr == 0 {
        return 0;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.bytes_sent.load(Ordering::Relaxed) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_getBytesReceived(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) -> jlong {
    if metrics_ptr == 0 {
        return 0;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    metrics.bytes_received.load(Ordering::Relaxed) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_getUptimeSeconds(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) -> jlong {
    if metrics_ptr == 0 {
        return 0;
    }
    
    let metrics = unsafe { &*(metrics_ptr as *const JavaRSocketPerformanceMetrics) };
    match metrics.start_time.elapsed() {
        Ok(duration) => duration.as_secs() as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_PerformanceMetrics_free(
    _env: JNIEnv,
    _class: JClass,
    metrics_ptr: jlong,
) {
    if metrics_ptr != 0 {
        unsafe {
            let _ = Box::from_raw(metrics_ptr as *mut JavaRSocketPerformanceMetrics);
        }
    }
}
