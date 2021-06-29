use pyo3::prelude::*;
use std::net::SocketAddr;

#[pyclass]
#[derive(Clone)]
pub struct SocketAddress {
    pub(crate) inner: SocketAddr,
}

impl SocketAddress {
    pub fn new(address: SocketAddr) -> Self {
        Self { inner: address }
    }
}

#[pymethods]
impl SocketAddress {
    #[new]
    pub fn __new__(address: String) -> PyResult<Self> {
        let inner = address.parse()?;
        Ok(Self { inner })
    }
}
