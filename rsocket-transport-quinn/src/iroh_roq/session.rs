use std::sync::Arc;
use anyhow::Result;
use iroh::endpoint::{Connection, VarInt};
use iroh_roq::{Session, SendFlow, ReceiveFlow};
use rtp::packet::Packet as RtpPacket;
use rsocket_rust::error::RSocketError;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct IrohRoqSessionConfig {
    pub max_flows: u32,
    pub recv_buffer_size: usize,
    pub enable_datagrams: bool,
    pub enable_streams: bool,
}

impl Default for IrohRoqSessionConfig {
    fn default() -> Self {
        Self {
            max_flows: 1000,
            recv_buffer_size: 64,
            enable_datagrams: true,
            enable_streams: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IrohRoqSession {
    session: Session,
    config: IrohRoqSessionConfig,
    flow_counter: Arc<Mutex<u32>>,
}

impl IrohRoqSession {
    pub fn new(conn: Connection, config: IrohRoqSessionConfig) -> Self {
        let session = Session::new(conn);
        Self {
            session,
            config,
            flow_counter: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn create_send_flow(&self) -> Result<SendFlow, RSocketError> {
        let mut counter = self.flow_counter.lock().await;
        let flow_id = VarInt::from_u32(*counter);
        *counter += 1;
        
        self.session.new_send_flow(flow_id)
            .await
            .map_err(|e| RSocketError::Other(e.into()))
    }

    pub async fn create_receive_flow(&self, flow_id: Option<VarInt>) -> Result<ReceiveFlow, RSocketError> {
        let id = match flow_id {
            Some(id) => id,
            None => {
                let mut counter = self.flow_counter.lock().await;
                let id = VarInt::from_u32(*counter);
                *counter += 1;
                id
            }
        };
        
        self.session.new_receive_flow(id)
            .await
            .map_err(|e| RSocketError::Other(e.into()))
    }

    pub async fn send_rtp_packet(&self, packet: &RtpPacket) -> Result<(), RSocketError> {
        let send_flow = self.create_send_flow().await?;
        send_flow.send_rtp(packet)
            .map_err(|e| RSocketError::Other(e.into()))
    }

    pub async fn receive_rtp_packet(&self, flow_id: VarInt) -> Result<RtpPacket, RSocketError> {
        let mut recv_flow = self.session.new_receive_flow(flow_id)
            .await
            .map_err(|e| RSocketError::Other(e.into()))?;
        
        recv_flow.read_rtp()
            .await
            .map_err(|e| RSocketError::Other(e.into()))
    }
}
