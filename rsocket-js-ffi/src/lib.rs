#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

mod simple_client;

use simple_client::SimpleRSocketClient;

#[napi]
pub fn create_simple_client() -> Result<SimpleRSocketClient> {
    Ok(SimpleRSocketClient::new())
}

#[napi]
pub fn get_supported_transports() -> Vec<String> {
    vec![
        "tcp".to_string(),
        "websocket".to_string(),
    ]
}

#[napi]
pub fn initialize_logger() -> Result<()> {
    env_logger::init();
    Ok(())
}

#[napi]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[napi]
pub fn get_library_info() -> HashMap<String, String> {
    let mut info = HashMap::new();
    info.insert("name".to_string(), "rsocket-rust-js".to_string());
    info.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    info.insert("description".to_string(), "JavaScript/Node.js bindings for RSocket Rust".to_string());
    info
}
