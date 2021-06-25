use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

#[path = "./common/server.rs"]
mod server;
#[path = "./common/socket.rs"]
mod socket;

mod socks6;
use socks6::*;

#[pymodule]
fn socksx(
    py: Python,
    m: &PyModule,
) -> PyResult<()> {
    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;

    // First initialize the Tokio runtime.
    pyo3_asyncio::try_init(py)?;
    pyo3_asyncio::tokio::init_multi_thread_once();

    m.add_wrapped(wrap_pymodule!(socks6))?;
    sys_modules.set_item("socksx.socks6", m.getattr("socks6")?)?;

    // Wrappers
    m.add_class::<socket::Socket>()?;
    m.add_class::<socket::SocketAddress>()?;
    m.add_class::<server::TcpServer>()?;

    Ok(())
}
