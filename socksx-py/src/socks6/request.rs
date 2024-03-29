use crate::socket::SocketAddress;
use crate::socks6::chain::Chain;
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use socksx::socks6::Socks6Request;
use socksx::{Address, ProxyAddress};
use std::convert::TryFrom;

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
    fn chain(
        &mut self,
        static_links: Option<Vec<String>>,
    ) -> PyResult<Option<Chain>> {
        let static_links: Vec<ProxyAddress> = static_links
            .unwrap_or_default()
            .into_iter()
            .map(|sl| ProxyAddress::try_from(sl).unwrap())
            .collect();

        self.inner
            .chain(&static_links)
            .map_err(|_| PyOSError::new_err("TODO: custom errors"))
            .map(|c| c.map(Chain::new))
    }

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
