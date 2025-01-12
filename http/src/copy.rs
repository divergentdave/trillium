use futures_lite::{io::BufReader, ready, AsyncBufRead, AsyncRead, AsyncWrite};
use std::{
    future::Future,
    io::{ErrorKind, Result},
    pin::Pin,
    task::{Context, Poll},
};

// to be tuned, not sure exactly how
const MAX_LOOPS: usize = 16;

pub async fn copy<R, W>(reader: R, writer: W) -> Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    struct CopyFuture<R, W> {
        reader: BufReader<R>,
        writer: W,
        amt: u64,
    }

    impl<R, W> Future for CopyFuture<R, W>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        type Output = Result<u64>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            for loop_number in 0..MAX_LOOPS {
                log::trace!("copy loop number: {loop_number}");
                let CopyFuture {
                    reader,
                    writer,
                    amt,
                } = &mut *self;

                let writer = Pin::new(writer);
                let mut reader = Pin::new(reader);
                let buffer = ready!(reader.as_mut().poll_fill_buf(cx))?;
                if buffer.is_empty() {
                    ready!(writer.poll_flush(cx))?;
                    return Poll::Ready(Ok(self.amt));
                }

                let i = ready!(writer.poll_write(cx, buffer))?;
                if i == 0 {
                    return Poll::Ready(Err(ErrorKind::WriteZero.into()));
                }
                *amt += i as u64;
                reader.consume(i);
            }

            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    let future = CopyFuture {
        reader: BufReader::new(reader),
        writer,
        amt: 0,
    };
    future.await
}
