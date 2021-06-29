use pyo3::prelude::*;

mod address;
mod function;
mod server;
mod socket;

pub use address::SocketAddress;
pub use function::SocketFunctionBuf;
pub use server::SocketServer;
pub use socket::Socket;

#[pymodule]
fn socket(
    _py: Python,
    m: &PyModule,
) -> PyResult<()> {
    // Wrappers
    m.add_class::<address::SocketAddress>()?;
    m.add_class::<function::SocketFunction>()?;
    m.add_class::<server::SocketServer>()?;
    m.add_class::<socket::Socket>()?;

    Ok(())
}
