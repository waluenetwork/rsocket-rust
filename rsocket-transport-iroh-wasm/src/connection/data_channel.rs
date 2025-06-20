use wasm_bindgen::prelude::*;
use web_sys::{RtcDataChannel, MessageEvent, ErrorEvent};
use futures_channel::mpsc;
use futures_util::StreamExt;
use js_sys::Uint8Array;


#[derive(Debug)]
pub struct IrohWasmDataChannel {
    data_channel: RtcDataChannel,
    message_sender: mpsc::UnboundedSender<Vec<u8>>,
    message_receiver: mpsc::UnboundedReceiver<Vec<u8>>,
}

impl IrohWasmDataChannel {
    pub fn new(data_channel: RtcDataChannel) -> Result<Self, JsValue> {
        let (message_sender, message_receiver) = mpsc::unbounded();
        
        let sender_clone = message_sender.clone();
        let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(array_buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                let uint8_array = Uint8Array::new(&array_buffer);
                let mut data = vec![0u8; uint8_array.length() as usize];
                uint8_array.copy_to(&mut data);
                
                log::debug!("📨 Received {} bytes via Iroh WASM data channel", data.len());
                
                if let Err(e) = sender_clone.unbounded_send(data) {
                    log::error!("❌ Failed to forward received data: {:?}", e);
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        data_channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
        
        let onerror = Closure::wrap(Box::new(move |event: ErrorEvent| {
            log::error!("❌ Iroh WASM data channel error: {:?}", event.message());
        }) as Box<dyn FnMut(_)>);
        
        data_channel.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
        
        let onopen = Closure::wrap(Box::new(move || {
            log::info!("✅ Iroh WASM data channel opened");
        }) as Box<dyn FnMut()>);
        
        data_channel.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();
        
        let onclose = Closure::wrap(Box::new(move || {
            log::info!("🔒 Iroh WASM data channel closed");
        }) as Box<dyn FnMut()>);
        
        data_channel.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();
        
        Ok(Self {
            data_channel,
            message_sender,
            message_receiver,
        })
    }
    
    pub fn send_bytes(&self, data: &[u8]) -> Result<(), JsValue> {
        self.data_channel.send_with_u8_array(data)?;
        log::debug!("📤 Sent {} bytes via Iroh WASM data channel", data.len());
        
        Ok(())
    }
    
    pub fn is_open(&self) -> bool {
        self.data_channel.ready_state() == web_sys::RtcDataChannelState::Open
    }
    
    pub fn close(&self) {
        self.data_channel.close();
    }
    
    pub fn split(self) -> (IrohWasmDataChannelSink, IrohWasmDataChannelStream) {
        let sink = IrohWasmDataChannelSink {
            data_channel: self.data_channel.clone(),
        };
        
        let stream = IrohWasmDataChannelStream {
            data_channel: self.data_channel,
            message_receiver: self.message_receiver,
        };
        
        (sink, stream)
    }
}

pub struct IrohWasmDataChannelSink {
    data_channel: RtcDataChannel,
}

pub struct IrohWasmDataChannelStream {
    data_channel: RtcDataChannel,
    message_receiver: mpsc::UnboundedReceiver<Vec<u8>>,
}


impl IrohWasmDataChannelSink {
    pub async fn send(&mut self, frame: Vec<u8>) -> Result<(), JsValue> {
        self.data_channel.send_with_u8_array(&frame)?;
        log::debug!("📤 Sent {} bytes via Iroh WASM data channel sink", frame.len());
        
        Ok(())
    }
}

impl IrohWasmDataChannelStream {
    pub async fn next(&mut self) -> Option<Result<Vec<u8>, JsValue>> {
        match self.message_receiver.next().await {
            Some(data) => {
                log::debug!("📨 Received {} bytes via Iroh WASM data channel stream", data.len());
                Some(Ok(data))
            }
            None => {
                log::debug!("🔚 Iroh WASM data channel stream ended");
                None
            }
        }
    }
}
