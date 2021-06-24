use std::os::unix::prelude::AsRawFd;

use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use socksx::Socks6Client;
use tokio::net::TcpStream;

#[pyclass]
#[derive(Clone)]
pub struct Client {
    pub proxy_addr: String,
}

#[pymethods]
impl Client {
    #[new]
    fn new(proxy_addr: String) -> Self {
        Self { proxy_addr }
    }

    fn connect(&mut self, destination: String, py: Python) -> PyResult<PyObject> {
        let proxy_addr = self.proxy_addr.clone();
        let destination = destination.clone();
    
        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let client = Socks6Client::new(proxy_addr, None).await
                .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;
    
            let (socket, _) = client.connect(destination, None, None).await
                .map_err(|_| PyOSError::new_err("TODO: custom errors"))?;
    
            let gil = Python::acquire_gil();
            let py = gil.python();
    
            let socket = Socket::new(socket).into_py(py);
            Ok(socket)
        })
    }  
}

#[pyclass]
pub struct Socket {
    inner: TcpStream
}

impl Socket {
    fn new(socket: TcpStream) -> Self {
        Self { inner: socket }
    }
}

#[pymethods]
impl Socket {
    fn get_raw_fd(&mut self) -> i32 {
        self.inner.as_raw_fd()
    }
}