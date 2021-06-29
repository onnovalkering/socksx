use crate::socket::Socket;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use std::ops::DerefMut;

mod socket;
use socket::*;

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

    // `socket` module
    m.add_wrapped(wrap_pymodule!(socket))?;
    sys_modules.set_item("socksx.socket", m.getattr("socket")?)?;

    // `socks6` module
    m.add_wrapped(wrap_pymodule!(socks6))?;
    sys_modules.set_item("socksx.socks6", m.getattr("socks6")?)?;

    // Utilities
    m.add_function(wrap_pyfunction!(copy_bidirectional, m)?)?;

    Ok(())
}

#[pyfunction]
pub fn copy_bidirectional(
    py: Python,
    a: &mut Socket,
    b: &mut Socket,
) -> PyResult<PyObject> {
    let a_tcp = a.inner.clone();
    let b_tcp = b.inner.clone();

    let a_fn = a.function.clone();
    let b_fn = b.function.clone();

    pyo3_asyncio::tokio::into_coroutine(py, async move {
        let mut a_tcp = a_tcp.write().await;
        let mut b_tcp = b_tcp.write().await;

        let mut a = SocketFunctionBuf::new(a_tcp.deref_mut(), a_fn);
        let mut b = SocketFunctionBuf::new(b_tcp.deref_mut(), b_fn);

        socksx::copy_bidirectional(&mut a, &mut b)
            .await
            .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;

        let gil = Python::acquire_gil();
        Ok(gil.python().None())
    })
}
