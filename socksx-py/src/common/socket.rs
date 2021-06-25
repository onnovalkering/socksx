use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use std::fmt::{self, Display, Formatter};
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

#[pyclass]
pub struct Socket {
    pub(crate) inner: Arc<RwLock<TcpStream>>,
}

impl Socket {
    pub fn new(socket: TcpStream) -> Self {
        let inner = Arc::new(RwLock::new(socket));
        Self { inner }
    }
}

#[pymethods]
impl Socket {
    pub fn copy_bidirectional(
        &mut self,
        py: Python,
        other: &mut Socket,
    ) -> PyResult<PyObject> {
        let inner = self.inner.clone();
        let other = other.inner.clone();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let mut inner = inner.write().await;
            let mut other = other.write().await;

            socksx::copy_bidirectional(&mut inner.deref_mut(), &mut other.deref_mut())
                .await
                .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;

            let gil = Python::acquire_gil();
            Ok(gil.python().None())
        })
    }

    pub fn get_original_dst(
        &mut self,
        py: Python,
    ) -> PyResult<PyObject> {
        let inner = self.inner.clone();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let raw_fd = inner.read().await.as_raw_fd();
            let original_dst = socksx::get_original_dst(&raw_fd)
                .map_err(|_| PyOSError::new_err("TODO: custom errors"))
                .map(SocketAddress::new)?;

            Ok(Python::with_gil(|gil| original_dst.into_py(gil)))
        })
    }

    pub fn get_raw_fd(
        &mut self,
        py: Python,
    ) -> PyResult<PyObject> {
        let inner = self.inner.clone();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let raw_fd = inner.read().await.as_raw_fd();
            Ok(Python::with_gil(|gil| raw_fd.into_py(gil)))
        })
    }

    pub fn try_read_initial_data(
        &mut self,
        py: Python,
    ) -> PyResult<PyObject> {
        let inner = self.inner.clone();

        // TODO: try to use socksx::try_read_initial_data.
        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let mut initial_data = Vec::with_capacity(2usize.pow(14)); // 16KB is the max
            inner.read().await.readable().await?;

            let bytes: Vec<u8> = match inner.read().await.try_read_buf(&mut initial_data) {
                Ok(0) => vec![],
                Ok(_) => initial_data,
                Err(e) => {
                    return Err(e.into());
                }
            };

            let gil = Python::acquire_gil();
            let py = gil.python();

            let bytes = bytes.into_py(py);
            Ok(bytes)
        })
    }
}

#[pyclass]
pub struct SocketAddress {
    pub(crate) inner: SocketAddr,
}

impl SocketAddress {
    pub fn new(addr: SocketAddr) -> Self {
        Self { inner: addr }
    }
}

impl Display for SocketAddress {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result {
        self.inner.fmt(f)
    }
}
