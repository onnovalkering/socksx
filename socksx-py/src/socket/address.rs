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
