use crate::error::box_error;
use crate::tokio::error::Listen;
use crate::tokio::Connection as TokioConnection;
use crate::{Connection, Framer};
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};
use snafu::ResultExt;
use std::marker::PhantomData;
use std::marker::Unpin;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::net::TcpListener;

pub struct Listener<F, S, R> {
    limit: Option<usize>,
    framer: F,
    inner: TcpListener,
    local: SocketAddr,
    remote: SocketAddr,
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
}

impl<F, S, R> Stream for Listener<F, S, R>
where
    F: Framer<S, R> + Clone + Unpin,
    S: Send + 'static,
    R: Send + 'static,
{
    type Item = Result<Box<dyn Connection<F, S, R>>, crate::error::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.as_mut().inner.incoming().poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(op) => Poll::Ready(match op {
                None => None,
                Some(res) => Some(
                    res.with_context(|| Listen)
                        .and_then(|raw| {
                            let remote = raw.peer_addr().with_context(|| Listen)?;
                            TokioConnection::from_existing(raw, self.framer.clone(), remote)
                        })
                        .map_err(box_error)
                        .with_context(|| crate::error::Listen),
                ),
            }),
        }
    }
}

impl<F, S, R> crate::Listener<F, S, R> for Listener<F, S, R>
where
    F: Framer<S, R> + Clone + Unpin,
    S: Send + 'static,
    R: Send + 'static,
{
    fn connection_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    fn local_endpoint(&self) -> SocketAddr {
        self.local
    }

    fn remote_endpoint(&self) -> SocketAddr {
        self.remote
    }
}
