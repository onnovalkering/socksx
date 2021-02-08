/// While this code has not been merged into tokio, include a copy here.
/// https://github.com/tokio-rs/tokio/pull/3230
/// MIT license; Copyright (c) Tokio Contributors
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

#[derive(Debug)]
pub(super) struct CopyBuffer {
    read_done: bool,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: Box<[u8]>,
}

impl CopyBuffer {
    pub(super) fn new() -> Self {
        Self {
            read_done: false,
            pos: 0,
            cap: 0,
            amt: 0,
            buf: vec![0; 2048].into_boxed_slice(),
        }
    }

    pub(super) fn poll_copy<R, W>(
        &mut self,
        cx: &mut Context<'_>,
        mut reader: Pin<&mut R>,
        mut writer: Pin<&mut W>,
    ) -> Poll<io::Result<u64>>
    where
        R: AsyncRead + ?Sized,
        W: AsyncWrite + ?Sized,
    {
        loop {
            // If our buffer is empty, then we need to read some data to
            // continue.
            if self.pos == self.cap && !self.read_done {
                let me = &mut *self;
                let mut buf = ReadBuf::new(&mut me.buf);
                ready!(reader.as_mut().poll_read(cx, &mut buf))?;
                let n = buf.filled().len();
                if n == 0 {
                    self.read_done = true;
                } else {
                    self.pos = 0;
                    self.cap = n;
                }
            }

            // If our buffer has some data, let's write it out!
            while self.pos < self.cap {
                let me = &mut *self;
                let i = ready!(writer.as_mut().poll_write(cx, &me.buf[me.pos..me.cap]))?;
                if i == 0 {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "write zero byte into writer",
                    )));
                } else {
                    self.pos += i;
                    self.amt += i as u64;
                }
            }

            // If we've written all the data and we've seen EOF, flush out the
            // data and finish the transfer.
            if self.pos == self.cap && self.read_done {
                ready!(writer.as_mut().poll_flush(cx))?;
                return Poll::Ready(Ok(self.amt));
            }
        }
    }
}

enum TransferState {
    Running(CopyBuffer),
    ShuttingDown(u64),
    Done(u64),
}

struct CopyBidirectional<'a, A: ?Sized, B: ?Sized> {
    a: &'a mut A,
    b: &'a mut B,
    a_to_b: TransferState,
    b_to_a: TransferState,
}

fn transfer_one_direction<A, B>(
    cx: &mut Context<'_>,
    state: &mut TransferState,
    r: &mut A,
    w: &mut B,
) -> Poll<io::Result<u64>>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    let mut r = Pin::new(r);
    let mut w = Pin::new(w);

    loop {
        match state {
            TransferState::Running(buf) => {
                *state = TransferState::ShuttingDown(ready!(buf.poll_copy(cx, r.as_mut(), w.as_mut()))?);
            }
            TransferState::ShuttingDown(count) => {
                ready!(w.as_mut().poll_shutdown(cx))?;

                *state = TransferState::Done(*count);
            }
            TransferState::Done(count) => break Poll::Ready(Ok(*count)),
        }
    }
}

fn swap_result<R, E>(r: Poll<Result<R, E>>) -> Result<Poll<R>, E> {
    match r {
        Poll::Pending => Ok(Poll::Pending),
        Poll::Ready(Ok(r)) => Ok(Poll::Ready(r)),
        Poll::Ready(Err(e)) => Err(e),
    }
}

impl<'a, A, B> std::future::Future for CopyBidirectional<'a, A, B>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    type Output = io::Result<(u64, u64)>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        // Unpack self into mut refs to each field to avoid borrow check issues.
        let CopyBidirectional { a, b, a_to_b, b_to_a } = &mut *self;

        let a_to_b = swap_result(transfer_one_direction(cx, a_to_b, &mut *a, &mut *b))?;
        let b_to_a = swap_result(transfer_one_direction(cx, b_to_a, &mut *b, &mut *a))?;

        let a_to_b = ready!(a_to_b);
        let b_to_a = ready!(b_to_a);

        Ok((a_to_b, b_to_a)).into()
    }
}

/// Copies data in both directions between `a` and `b`.
///
/// This function returns a future which will read from both streams, and
/// write any data read to the opposing stream. The data transfer in one
/// direction (eg, `a` to `b`) will not be blocked if the opposite direction
/// blocks (e.g. if writing from `b` to `a` is blocked, `a` to `b` will not
/// be blocked).
///
/// If an EOF is observed on one stream, [`shutdown()`] will be invoked on
/// the other, and reading from that stream will stop. Copying of data in
/// the other direction will continue.
///
/// The future will complete successfully once both streams return EOF,
/// returning a tuple of the number of bytes copied from `a` to `b` and the
/// number of bytes copied from `b` to `a`, in that order.
///
/// [`shutdown()`]: crate::io::AsyncWriteExt::shutdown
///
/// # Errors
///
/// The future will immediately return an error if any IO operation on `a`
/// or `b` returns an error. Some data read from either stream may be lost (not
/// written to the other stream) in this case.
///
/// # Return value
///
/// Returns a tuple of bytes copied `a` to `b` and bytes copied `b` to `a`.
pub async fn copy_bidirectional<A, B>(
    a: &mut A,
    b: &mut B,
) -> Result<(u64, u64), std::io::Error>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    CopyBidirectional {
        a,
        b,
        a_to_b: TransferState::Running(CopyBuffer::new()),
        b_to_a: TransferState::Running(CopyBuffer::new()),
    }
    .await
}