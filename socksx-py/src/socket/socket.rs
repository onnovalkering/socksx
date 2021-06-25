use crate::socket::SocketAddress;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

#[pyclass]
pub struct Socket {
    pub(crate) inner: Arc<RwLock<TcpStream>>,
}

impl Socket {
    ///
    ///
    ///
    pub fn new(socket: TcpStream) -> Self {
        let inner = Arc::new(RwLock::new(socket));
        Self { inner }
    }
}

#[pymethods]
impl Socket {
    ///
    ///
    ///
    #[staticmethod]
    pub fn connect(
        py: Python,
        address: SocketAddress,
    ) -> PyResult<PyObject> {
        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let socket = TcpStream::connect(address.inner)
                .await
                .map_err(|_| PyOSError::new_err("TODO: custom errors"))
                .map(Socket::new)?;

            Ok(Python::with_gil(|gil| socket.into_py(gil)))
        })
    }

    ///
    ///
    ///
    pub fn flush(
        &mut self,
        py: Python,
    ) -> PyResult<PyObject> {
        let inner = self.inner.clone();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            inner
                .write()
                .await
                .flush()
                .await
                .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;

            let gil = Python::acquire_gil();
            Ok(gil.python().None())
        })
    }

    ///
    ///
    ///    
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

    ///
    ///
    ///    
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

    ///
    ///
    ///    
    pub fn try_read_initial_data(
        &mut self,
        py: Python,
    ) -> PyResult<PyObject> {
        let inner = self.inner.clone();
        
        pyo3_asyncio::tokio::into_coroutine(py, async move {
            // TODO: try to use socksx::try_read_initial_data.
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
