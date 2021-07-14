use crate::socket::Socket;
use crate::socket::SocketAddress;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use socksx::Socks6Client;

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

    fn connect(
        &mut self,
        py: Python,
        destination: SocketAddress,
    ) -> PyResult<PyObject> {
        let proxy_addr = self.proxy_addr.clone();
        let destination = destination.inner.to_string();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let client = Socks6Client::new(proxy_addr, None)
                .await
                .map_err(|e| PyOSError::new_err(e.to_string()))?;

            let (socket, _) = client
                .connect(destination, None, None)
                .await
                .map_err(|e| PyOSError::new_err(e.to_string()))?;

            let gil = Python::acquire_gil();
            let py = gil.python();

            let socket = Socket::new(socket).into_py(py);
            Ok(socket)
        })
    }
}
