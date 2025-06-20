use log::{debug, info, warn, error};
use super::{TransportType, TransportCapability};

pub struct TransportLogger;

impl TransportLogger {
    pub fn log_transport_selection(selected: &TransportType, available: &[TransportType]) {
        info!("Selected transport: {:?}", selected);
        debug!("Available transports: {:?}", available);
    }

    pub fn log_transport_fallback(from: &TransportType, to: &TransportType, reason: &str) {
        warn!("Transport fallback: {:?} -> {:?}, reason: {}", from, to, reason);
    }

    pub fn log_capability_detection(capabilities: &[TransportCapability]) {
        debug!("Detected platform capabilities: {:?}", capabilities);
    }

    pub fn log_transport_error(transport: &TransportType, error: &str) {
        error!("Transport {:?} error: {}", transport, error);
    }

    pub fn log_platform_detection(platform: &str, supported_transports: &[TransportType]) {
        info!("Platform: {}, supported transports: {:?}", platform, supported_transports);
    }
}

#[macro_export]
macro_rules! transport_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "transport-debug")]
        log::debug!("[TRANSPORT] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! transport_info {
    ($($arg:tt)*) => {
        log::info!("[TRANSPORT] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! transport_warn {
    ($($arg:tt)*) => {
        log::warn!("[TRANSPORT] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! transport_error {
    ($($arg:tt)*) => {
        log::error!("[TRANSPORT] {}", format!($($arg)*));
    };
}
