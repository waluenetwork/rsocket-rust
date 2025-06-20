#![allow(clippy::type_complexity)]

extern crate log;

pub mod client;
pub mod connection;
pub mod misc;

#[cfg(feature = "webworkers")]
pub mod webworkers;

pub use client::IrohWasmClientTransport;
pub use connection::{IrohWasmConnection, IrohWasmConnectionStats};
pub use misc::{IrohWasmConfig, IrohWasmCapabilities, detect_iroh_wasm_capabilities, is_webrtc_supported};

#[cfg(feature = "webworkers")]
pub use webworkers::{
    IrohWasmWebWorkersTransport, IrohWasmWebWorkersConnection, IrohWasmWebWorkersConfig,
    IrohWasmPerformanceMonitor, IrohWasmPerformanceMetrics, IrohWasmWorkerPool,
    create_iroh_wasm_optimized_config, benchmark_iroh_wasm_performance
};

pub mod frame {
    use bytes::Bytes;
    
    #[derive(Debug, Clone)]
    pub struct Frame {
        pub payload: Bytes,
        pub frame_type: FrameType,
    }
    
    #[derive(Debug, Clone, Copy)]
    pub enum FrameType {
        Setup,
        RequestResponse,
        RequestStream,
        RequestChannel,
        RequestFnf,
        Payload,
        Error,
        Cancel,
        KeepAlive,
    }
    
    impl Frame {
        pub fn new(frame_type: FrameType, payload: Bytes) -> Self {
            Self { payload, frame_type }
        }
        
        pub fn payload(&self) -> &Bytes {
            &self.payload
        }
        
        pub fn frame_type(&self) -> FrameType {
            self.frame_type
        }
    }
}
