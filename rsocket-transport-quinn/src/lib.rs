#![allow(clippy::type_complexity)]

#[cfg(not(target_arch = "wasm32"))]
mod client;
#[cfg(not(target_arch = "wasm32"))]
mod connection;
#[cfg(not(target_arch = "wasm32"))]
mod misc;
#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(feature = "webtransport")]
pub mod webtransport;

#[cfg(feature = "iroh-roq")]
pub mod iroh_roq;

#[cfg(not(target_arch = "wasm32"))]
pub use client::QuinnClientTransport;
#[cfg(not(target_arch = "wasm32"))]
pub use connection::QuinnConnection;
#[cfg(not(target_arch = "wasm32"))]
pub use server::QuinnServerTransport;

#[cfg(feature = "webtransport")]
pub use webtransport::{WebTransportClientTransport, WebTransportConnection, WebTransportServerTransport};

#[cfg(feature = "iroh-roq")]
pub use iroh_roq::{
    IrohRoqClientTransport, IrohRoqServerTransport, IrohRoqConnection, 
    IrohRoqSession, IrohRoqSessionConfig
};
