mod fragmentation;
mod misc;
mod socket;
mod spi;
pub mod crossbeam_socket;
pub mod crossbeam_integration;
pub mod simd_frame_processing;

pub(crate) use fragmentation::{Joiner, Splitter, MIN_MTU};
pub(crate) use socket::{ClientRequester,DuplexSocket};
pub use spi::*;
pub use crossbeam_socket::{CrossbeamDuplexSocket, CrossbeamClientRequester, CrossbeamServerRequester};
pub use crossbeam_integration::CrossbeamOptimizedSocket;
pub use simd_frame_processing::SimdFrameProcessor;
