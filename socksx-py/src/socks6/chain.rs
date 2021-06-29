use pyo3::prelude::*;
use socksx::ProxyAddress;
use socksx::socks6::SocksChain;

#[pyclass]
#[derive(Clone)]
pub struct Chain {
    pub(crate) inner: SocksChain
}

impl Chain {
    pub fn new(chain: SocksChain) -> Self {
        Self {
            inner: chain
        }
    }
}

#[pymethods]
impl Chain {
    ///
    ///
    ///
    pub fn has_next(&self) -> bool {
        self.inner.has_next()
    }

    ///
    ///
    ///
    pub fn next_link(&mut self) -> Option<ChainLink> {
        self.inner.next_link().cloned().map(ChainLink::new)
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ChainLink {
    pub(crate) inner: ProxyAddress
}

impl ChainLink {
    ///
    ///
    ///
    pub fn new(inner: ProxyAddress) -> Self {
        Self {
            inner
        }
    }
}

#[pymethods]
impl ChainLink {
    #[getter]
    fn host(&self) -> String {
        self.inner.host.clone()
    }

    #[getter]
    fn port(&self) -> u16 {
        self.inner.port
    }    
}
