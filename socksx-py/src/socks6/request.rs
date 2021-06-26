use crate::socket::SocketAddress;
use pyo3::prelude::*;
use socksx::socks6::Socks6Request;
use socksx::Address;

#[pyclass]
#[derive(Clone)]
pub struct Request {
    pub(crate) inner: Socks6Request,
}

impl Request {
    pub fn new(request: Socks6Request) -> Self {
        Self { inner: request }
    }
}

#[pymethods]
impl Request {
    #[getter]
    fn command(&self) -> PyResult<u8> {
        Ok(self.inner.command.clone() as u8)
    }

    #[getter]
    fn destination(&self) -> PyResult<SocketAddress> {
        if let Address::Ip(socket_addr) = self.inner.destination {
            Ok(SocketAddress::new(socket_addr))
        } else {
            todo!()
        }
    }

    #[getter]
    fn initial_data_length(&self) -> PyResult<u16> {
        Ok(self.inner.initial_data_length)
    }
}
