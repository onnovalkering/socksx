use pyo3::prelude::*;

mod client;

#[pymodule]
fn socks6(
    _py: Python,
    m: &PyModule,
) -> PyResult<()> {
    m.add_class::<client::Client>()?;

    Ok(())
}
