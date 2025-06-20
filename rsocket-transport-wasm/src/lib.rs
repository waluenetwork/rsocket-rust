#![allow(clippy::type_complexity)]
#![allow(clippy::unused_unit)]
#![allow(clippy::from_over_into)]

#[macro_use]
extern crate serde_derive;

mod client;
mod connection;
mod misc;

#[cfg(feature = "webworkers")]
pub mod webworkers;

#[cfg(all(feature = "wasm-only", feature = "webworkers"))]
pub use webworkers::simple_wasm::*;

pub use client::WebsocketClientTransport;
pub use connection::WebsocketConnection;
pub use misc::{connect, new_payload, JsClient, JsPayload};

#[cfg(feature = "webworkers")]
pub use webworkers::{
    WebWorkersClientTransport, 
    WebWorkersConfig, 
    PerformanceMonitor, 
    SharedRingBuffer,
    WorkerPool,
    RSocketWorker
};
