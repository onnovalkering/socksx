use crate::socket::Socket;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use std::sync::Arc;
use tokio::net::TcpListener;

#[pyclass]
pub struct SocketServer {
    _address: String,
    listener: Arc<TcpListener>,
}

#[pymethods]
impl SocketServer {
    #[staticmethod]
    pub fn bind(
        py: Python,
        host: String,
        port: u16,
    ) -> PyResult<PyObject> {
        let address = format!("{}:{}", host, port);

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let listener = TcpListener::bind(address.clone())
                .await
                .map_err(|_| PyOSError::new_err("TODO: custom errors"))
                .map(Arc::new)?;

            let server = SocketServer {
                _address: address,
                listener,
            };

            Ok(Python::with_gil(|gil| server.into_py(gil)))
        })
    }

    pub fn accept(
        &mut self,
        py: Python,
    ) -> PyResult<PyObject> {
        let listener = self.listener.clone();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let (socket, _) = listener.accept().await?;
            let socket = Socket::new(socket);

            Ok(Python::with_gil(|gil| socket.into_py(gil)))
        })
    }
}
