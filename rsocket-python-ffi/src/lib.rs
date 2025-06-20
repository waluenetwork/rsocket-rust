use pyo3::prelude::*;
// use rsocket_rust_transport_quinn::{QuinnClientTransport, QuinnServerTransport};
// use rsocket_rust_transport_iroh::{IrohClientTransport, IrohServerTransport};

mod client;
mod server;
mod transport;
mod payload;
mod factory;

use client::*;
use server::*;
use transport::*;
use payload::*;
use factory::*;

#[pymodule]
fn rsocket_rust_ffi(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyPayload>()?;
    m.add_class::<PyPayloadBuilder>()?;
    m.add_class::<PyClient>()?;
    
    m.add_class::<PyTcpClientTransport>()?;
    m.add_class::<PyTcpServerTransport>()?;
    m.add_class::<PyWebSocketClientTransport>()?;
    m.add_class::<PyWebSocketServerTransport>()?;
    m.add_class::<PyQuinnClientTransport>()?;
    m.add_class::<PyQuinnServerTransport>()?;
    m.add_class::<PyIrohClientTransport>()?;
    m.add_class::<PyIrohServerTransport>()?;
    
    #[cfg(feature = "advanced-transports")]
    {
        m.add_class::<PyWebTransportClientTransport>()?;
        m.add_class::<PyWebTransportServerTransport>()?;
        m.add_class::<PyWebWorkersClientTransport>()?;
        m.add_class::<PyWebWorkersConfig>()?;
    }
    
    m.add_class::<PyRSocketFactory>()?;
    m.add_class::<PyMultiTransportServerBuilder>()?;
    
    Ok(())
}
