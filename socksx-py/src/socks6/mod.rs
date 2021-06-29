use crate::socket::Socket;
use crate::socks6::request::Request;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use socksx;
use socksx::socks6::Socks6Reply;
use std::ops::DerefMut;

mod chain;
mod client;
mod request;

#[pymodule]
fn socks6(
    _py: Python,
    m: &PyModule,
) -> PyResult<()> {
    m.add_class::<chain::Chain>()?;
    m.add_class::<client::Client>()?;
    m.add_class::<request::Request>()?;

    m.add_function(wrap_pyfunction!(read_request, m)?)?;
    m.add_function(wrap_pyfunction!(write_no_authentication, m)?)?;
    m.add_function(wrap_pyfunction!(write_connection_refused_reply, m)?)?;
    m.add_function(wrap_pyfunction!(write_success_reply, m)?)?;

    Ok(())
}

#[pyfunction]
fn read_request(
    py: Python,
    socket: &Socket,
) -> PyResult<PyObject> {
    let stream = socket.inner.clone();

    pyo3_asyncio::tokio::into_coroutine(py, async move {
        let mut stream = stream.write().await;
        let request = socksx::socks6::read_request(stream.deref_mut())
            .await
            .map_err(|_| PyOSError::new_err("TODO: custom errors"))
            .map(Request::new)?;

        Ok(Python::with_gil(|gil| request.into_py(gil)))
    })
}

#[pyfunction]
fn write_no_authentication(
    py: Python,
    socket: &Socket,
) -> PyResult<PyObject> {
    let stream = socket.inner.clone();

    pyo3_asyncio::tokio::into_coroutine(py, async move {
        let mut stream = stream.write().await;
        socksx::socks6::write_no_authentication(stream.deref_mut())
            .await
            .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;

        let gil = Python::acquire_gil();
        Ok(gil.python().None())
    })
}

#[pyfunction]
fn write_connection_refused_reply(
    py: Python,
    socket: &Socket,
) -> PyResult<PyObject> {
    write_reply(py, socket, Socks6Reply::ConnectionRefused)
}

#[pyfunction]
fn write_success_reply(
    py: Python,
    socket: &Socket,
) -> PyResult<PyObject> {
    write_reply(py, socket, Socks6Reply::Success)
}

fn write_reply(
    py: Python,
    socket: &Socket,
    reply: Socks6Reply,
) -> PyResult<PyObject> {
    let stream = socket.inner.clone();

    pyo3_asyncio::tokio::into_coroutine(py, async move {
        let mut stream = stream.write().await;
        socksx::socks6::write_reply(stream.deref_mut(), reply)
            .await
            .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;

        let gil = Python::acquire_gil();
        Ok(gil.python().None())
    })
}
