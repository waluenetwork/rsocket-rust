use wasm_bindgen::prelude::*;
use web_sys::{RtcPeerConnection, RtcDataChannel, RtcSessionDescription};

use crate::{
    connection::IrohWasmConnection,
    misc::{IrohWasmConfig, establish_p2p_connection, log_iroh_wasm_capabilities},
};

#[derive(Debug)]
enum IrohWasmConnector {
    SignalingServer(String, IrohWasmConfig),
    DirectPeerConnection(RtcPeerConnection),
    PeerConnectionWithDataChannel(RtcPeerConnection, RtcDataChannel),
}

#[derive(Debug)]
pub struct IrohWasmClientTransport {
    connector: IrohWasmConnector,
}

impl IrohWasmClientTransport {
    pub fn new(signaling_server: String, config: IrohWasmConfig) -> Self {
        log_iroh_wasm_capabilities();
        
        Self {
            connector: IrohWasmConnector::SignalingServer(signaling_server, config),
        }
    }
    
    pub fn from_peer_connection(peer_connection: RtcPeerConnection) -> Self {
        Self {
            connector: IrohWasmConnector::DirectPeerConnection(peer_connection),
        }
    }
    
    pub fn from_peer_connection_with_data_channel(
        peer_connection: RtcPeerConnection,
        data_channel: RtcDataChannel,
    ) -> Self {
        Self {
            connector: IrohWasmConnector::PeerConnectionWithDataChannel(peer_connection, data_channel),
        }
    }
    
    async fn create_data_channel(peer_connection: &RtcPeerConnection) -> Result<RtcDataChannel, JsValue> {
        let data_channel = peer_connection.create_data_channel("iroh-rsocket");
        
        
        log::info!("📡 Created Iroh WASM data channel for RSocket communication");
        
        Ok(data_channel)
    }
    
    async fn perform_signaling_handshake(
        peer_connection: &RtcPeerConnection,
        _signaling_server: &str,
    ) -> Result<(), JsValue> {
        let _offer = wasm_bindgen_futures::JsFuture::from(peer_connection.create_offer())
            .await?
            .dyn_into::<RtcSessionDescription>()?;
        
        log::info!("📝 Would set local description for WebRTC offer");
        log::info!("🤝 Created WebRTC offer for Iroh P2P signaling");
        log::info!("✅ Simulated successful Iroh P2P signaling handshake");
        
        Ok(())
    }
}

impl IrohWasmClientTransport {
    pub async fn connect(self) -> Result<IrohWasmConnection, JsValue> {
        match self.connector {
            IrohWasmConnector::SignalingServer(signaling_server, config) => {
                log::info!("🔗 Establishing Iroh P2P connection via signaling server: {}", signaling_server);
                
                let peer_connection = establish_p2p_connection(&config, &signaling_server).await?;
                let data_channel = Self::create_data_channel(&peer_connection).await?;
                
                Self::perform_signaling_handshake(&peer_connection, &signaling_server).await?;
                
                let connection = IrohWasmConnection::new(peer_connection, data_channel)?;
                
                log::info!("✅ Iroh WASM P2P connection established successfully");
                Ok(connection)
            }
            
            IrohWasmConnector::DirectPeerConnection(peer_connection) => {
                log::info!("🔗 Using direct peer connection for Iroh WASM transport");
                
                let data_channel = Self::create_data_channel(&peer_connection).await?;
                let connection = IrohWasmConnection::new(peer_connection, data_channel)?;
                
                Ok(connection)
            }
            
            IrohWasmConnector::PeerConnectionWithDataChannel(peer_connection, data_channel) => {
                log::info!("✅ Using pre-configured peer connection and data channel");
                
                let connection = IrohWasmConnection::new(peer_connection, data_channel)?;
                Ok(connection)
            }
        }
    }
}

impl From<String> for IrohWasmClientTransport {
    fn from(signaling_server: String) -> Self {
        Self::new(signaling_server, IrohWasmConfig::default())
    }
}

impl From<&str> for IrohWasmClientTransport {
    fn from(signaling_server: &str) -> Self {
        Self::from(signaling_server.to_string())
    }
}

impl From<RtcPeerConnection> for IrohWasmClientTransport {
    fn from(peer_connection: RtcPeerConnection) -> Self {
        Self::from_peer_connection(peer_connection)
    }
}
