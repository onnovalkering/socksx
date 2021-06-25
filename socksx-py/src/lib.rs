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
    let a = a.inner.clone();
    let b = b.inner.clone();

    pyo3_asyncio::tokio::into_coroutine(py, async move {
        let mut a = a.write().await;
        let mut b = b.write().await;

        socksx::copy_bidirectional(&mut a.deref_mut(), &mut b.deref_mut())
            .await
            .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;

        let gil = Python::acquire_gil();
        Ok(gil.python().None())
    })
}
