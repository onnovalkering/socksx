use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

mod socks6;
use socks6::*;

#[pymodule]
fn socksx(py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_asyncio::try_init(py)?;
    pyo3_asyncio::tokio::init_multi_thread_once();
    
    m.add_wrapped(wrap_pymodule!(socks6))?;

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("socksx.socks6", m.getattr("socks6")?)?;

    Ok(())
}