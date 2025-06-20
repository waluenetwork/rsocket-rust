use wasm_bindgen::prelude::*;
use web_sys::{RtcPeerConnection, RtcDataChannel};

use super::data_channel::IrohWasmDataChannel;

#[derive(Debug)]
pub struct IrohWasmConnection {
    peer_connection: RtcPeerConnection,
    data_channel: IrohWasmDataChannel,
}

impl IrohWasmConnection {
    pub fn new(peer_connection: RtcPeerConnection, data_channel: RtcDataChannel) -> Result<Self, JsValue> {
        let iroh_data_channel = IrohWasmDataChannel::new(data_channel)?;
        
        Ok(Self {
            peer_connection,
            data_channel: iroh_data_channel,
        })
    }
    
    pub fn is_connected(&self) -> bool {
        self.peer_connection.connection_state() == web_sys::RtcPeerConnectionState::Connected
            && self.data_channel.is_open()
    }
    
    pub fn close(&self) {
        self.data_channel.close();
        self.peer_connection.close();
        log::info!("🔒 Closed Iroh WASM P2P connection");
    }
    
    pub fn get_connection_stats(&self) -> IrohWasmConnectionStats {
        IrohWasmConnectionStats {
            connection_state: format!("{:?}", self.peer_connection.connection_state()),
            data_channel_state: format!("{:?}", self.data_channel.is_open()),
            is_connected: self.is_connected(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct IrohWasmConnectionStats {
    pub connection_state: String,
    pub data_channel_state: String,
    pub is_connected: bool,
}
