use pin_project_lite::pin_project;
use pyo3::prelude::*;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncBufRead, BufReader, BufWriter};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pin_project! {
    #[derive(Debug)]
    pub struct SocketFunction<RW> {
        #[pin]
        inner: BufReader<BufWriter<RW>>,
        function: Option<PyObject>,
    }
}

impl<RW: AsyncRead + AsyncWrite> SocketFunction<RW> {
    pub fn new(
        stream: RW,
        function: Option<PyObject>,
    ) -> SocketFunction<RW> {
        SocketFunction {
            inner: BufReader::new(BufWriter::new(stream)),
            function,
        }
    }
}

impl<RW: AsyncRead + AsyncWrite> AsyncWrite for SocketFunction<RW> {
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

impl<RW: AsyncRead + AsyncWrite> AsyncRead for SocketFunction<RW> {
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

        // Apply function
        let data = if let Some(function) = &self.function {
            partial(function, data)?
        } else {
            data
        };

        buf.put_slice(&data[..]);
        self.as_mut().project().inner.consume(amt);

        Poll::Ready(Ok(()))
    }
}

pub fn partial(function: &PyObject, data: Vec<u8>) -> PyResult<Vec<u8>> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let data = function.call_method1(py, "partial", (&data[..],))?;
    let data: Vec<u8> = data.extract(py)?;

    Ok(data)
}