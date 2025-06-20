use wasm_bindgen::prelude::*;
use web_sys::{RtcConfiguration, RtcPeerConnection};
use js_sys::{Array, Object, Reflect};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrohWasmConfig {
    pub ice_servers: Vec<String>,
    pub enable_webworkers: bool,
    pub worker_count: usize,
    pub buffer_size: usize,
    pub enable_performance_monitoring: bool,
    pub connection_timeout_ms: u32,
    pub max_retries: u32,
}

impl Default for IrohWasmConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            enable_webworkers: true,
            worker_count: navigator_hardware_concurrency().unwrap_or(4),
            buffer_size: 1024 * 1024, // 1MB
            enable_performance_monitoring: false,
            connection_timeout_ms: 30000, // 30 seconds
            max_retries: 3,
        }
    }
}

pub fn create_webrtc_config(config: &IrohWasmConfig) -> Result<RtcConfiguration, JsValue> {
    let rtc_config = RtcConfiguration::new();
    
    let ice_servers = Array::new();
    for server_url in &config.ice_servers {
        let ice_server = Object::new();
        let urls = Array::new();
        urls.push(&JsValue::from_str(server_url));
        Reflect::set(&ice_server, &JsValue::from_str("urls"), &urls)?;
        ice_servers.push(&ice_server);
    }
    
    rtc_config.set_ice_servers(&ice_servers);
    
    Ok(rtc_config)
}

pub async fn establish_p2p_connection(
    config: &IrohWasmConfig,
    _signaling_server: &str,
) -> Result<RtcPeerConnection, JsValue> {
    let rtc_config = create_webrtc_config(config)?;
    let peer_connection = RtcPeerConnection::new_with_configuration(&rtc_config)?;
    
    log::info!("🔗 Created WebRTC peer connection for Iroh P2P transport");
    
    let _pc_clone = peer_connection.clone();
    let onconnectionstatechange = Closure::wrap(Box::new(move || {
        let state = _pc_clone.connection_state();
        log::info!("🔄 WebRTC connection state changed: {:?}", state);
    }) as Box<dyn FnMut()>);
    
    peer_connection.set_onconnectionstatechange(Some(onconnectionstatechange.as_ref().unchecked_ref()));
    onconnectionstatechange.forget();
    
    let onicecandidate = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        log::debug!("🧊 ICE candidate event received");
    }) as Box<dyn FnMut(_)>);
    
    peer_connection.set_onicecandidate(Some(onicecandidate.as_ref().unchecked_ref()));
    onicecandidate.forget();
    
    Ok(peer_connection)
}

pub fn navigator_hardware_concurrency() -> Option<usize> {
    use web_sys::window;
    window()
        .map(|w| w.navigator().hardware_concurrency() as usize)
}

pub fn is_webrtc_supported() -> bool {
    use js_sys::global;
    use wasm_bindgen::JsValue;
    
    let global = global();
    js_sys::Reflect::has(&global, &JsValue::from_str("RTCPeerConnection"))
        .unwrap_or(false)
}

pub fn detect_iroh_wasm_capabilities() -> IrohWasmCapabilities {
    IrohWasmCapabilities {
        webrtc_supported: is_webrtc_supported(),
        webworkers_supported: is_webworkers_supported(),
        shared_array_buffer_supported: is_shared_array_buffer_supported(),
        optimal_worker_count: navigator_hardware_concurrency().unwrap_or(4),
        max_buffer_size: if is_shared_array_buffer_supported() { 2 * 1024 * 1024 } else { 512 * 1024 },
    }
}

#[derive(Debug, Clone)]
pub struct IrohWasmCapabilities {
    pub webrtc_supported: bool,
    pub webworkers_supported: bool,
    pub shared_array_buffer_supported: bool,
    pub optimal_worker_count: usize,
    pub max_buffer_size: usize,
}

fn is_shared_array_buffer_supported() -> bool {
    use wasm_bindgen::JsValue;
    use js_sys::global;
    
    let global = global();
    js_sys::Reflect::has(&global, &JsValue::from_str("SharedArrayBuffer"))
        .unwrap_or(false)
}

fn is_webworkers_supported() -> bool {
    use wasm_bindgen::JsValue;
    use js_sys::global;
    
    let global = global();
    js_sys::Reflect::has(&global, &JsValue::from_str("Worker"))
        .unwrap_or(false)
}

pub const IROH_WASM_ALPN: &[u8] = b"iroh-wasm-rsocket";

pub fn log_iroh_wasm_capabilities() {
    let caps = detect_iroh_wasm_capabilities();
    log::info!("🔍 Iroh WASM Capabilities:");
    log::info!("  WebRTC: {}", caps.webrtc_supported);
    log::info!("  WebWorkers: {}", caps.webworkers_supported);
    log::info!("  SharedArrayBuffer: {}", caps.shared_array_buffer_supported);
    log::info!("  Optimal Workers: {}", caps.optimal_worker_count);
}
