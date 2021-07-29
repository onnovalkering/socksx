use pin_project_lite::pin_project;
use pyo3::prelude::*;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncBufRead, BufReader, BufWriter};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[pyclass(subclass)]
pub struct SocketFunction {}

#[pymethods]
impl SocketFunction {
    #[new]
    pub fn __new__() -> Self {
        Self {}
    }

    pub fn partial(
        &mut self,
        data: Vec<u8>,
    ) -> PyResult<Vec<u8>> {
        Ok(data)
    }

    pub fn end(&mut self) {}
}

pin_project! {
    #[derive(Debug)]
    pub struct SocketFunctionBuf<RW> {
        #[pin]
        inner: BufReader<BufWriter<RW>>,
        function: Option<PyObject>,
    }
}

impl<RW: AsyncRead + AsyncWrite> SocketFunctionBuf<RW> {
    pub fn new(
        stream: RW,
        function: Option<PyObject>,
    ) -> SocketFunctionBuf<RW> {
        SocketFunctionBuf {
            inner: BufReader::new(BufWriter::new(stream)),
            function,
        }
    }
}

impl<RW: AsyncRead + AsyncWrite> AsyncWrite for SocketFunctionBuf<RW> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().inner.poll_shutdown(cx)
    }
}

impl<RW: AsyncRead + AsyncWrite> AsyncRead for SocketFunctionBuf<RW> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let reader = self.as_mut().project().inner;

        let remaining = match reader.poll_fill_buf(cx) {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }?;

        let amt = std::cmp::min(remaining.len(), buf.remaining());
        let data = remaining[..amt].to_vec();

        // Apply function, if any, to data.
        let data = if let Some(function) = &self.function {
            if data.is_empty() {
                // If data is empty, we reached EOF.
                end(function)?;
                return Poll::Ready(Ok(()));
            }

            partial(function, data)?
        } else {
            data
        };

        buf.put_slice(&data[..]);
        self.as_mut().project().inner.consume(amt);

        Poll::Ready(Ok(()))
    }
}

///
///
///
pub fn partial(
    function: &PyObject,
    data: Vec<u8>,
) -> PyResult<Vec<u8>> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let data = function.call_method1(py, "partial", (&data[..],))?;
    let data: Vec<u8> = data.extract(py)?;

    Ok(data)
}

///
///
///
pub fn end(function: &PyObject) -> PyResult<()> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    function.call_method1(py, "end", ())?;
    Ok(())
}
