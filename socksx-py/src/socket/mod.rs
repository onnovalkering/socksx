use pyo3::prelude::*;

mod address;
mod server;
mod socket;

pub use address::SocketAddress;
pub use server::SocketServer;
pub use socket::Socket;

#[pymodule]
fn socket(
    _py: Python,
    m: &PyModule,
) -> PyResult<()> {
    // Wrappers
    m.add_class::<address::SocketAddress>()?;
    m.add_class::<server::SocketServer>()?;
    m.add_class::<socket::Socket>()?;

    Ok(())
}
