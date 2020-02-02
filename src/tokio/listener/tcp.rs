use crate::tokio::error::Listen;
use crate::tokio::Connection as TokioConnection;
use crate::{Connection, Framer};
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};
use snafu::ResultExt;
use std::marker::Unpin;
use std::pin::Pin;
use tokio::net::TcpListener;
use crate::error::box_error;

pub struct Listener<F> {
    limit: Option<usize>,
    framer: F,
    inner: TcpListener,
}

impl<F> Stream for Listener<F>
where
    F: Framer + Send + 'static + Clone + Unpin,
    F::Input: Send,
{
    type Item = Result<
        Box<dyn Connection<F>>,
        crate::error::Error,
    >;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().inner.incoming().poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(op) => Poll::Ready(match op {
                None => None,
                Some(res) => Some(
                    res.with_context(|| Listen)
                        .map(|raw| TokioConnection::from_existing(raw, self.framer.clone()))
                        .map_err(box_error)
                        .with_context(|| crate::error::Listen),
                ),
            }),
        }
    }
}

impl<F> crate::Listener<F> for Listener<F>
where
    F: Framer + Clone + Unpin,
    F::Input: Send,
{
    fn connection_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }
}
