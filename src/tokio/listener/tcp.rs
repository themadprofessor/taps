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

pub struct Listener<F> {
    limit: Option<usize>,
    framer: F,
    inner: TcpListener,
    local: SocketAddr,
    remote: SocketAddr,
}

impl<F> Stream for Listener<F>
where
    F: Framer + Clone + Unpin,
{
    type Item = Result<Box<dyn Connection<F>>, crate::error::Error>;

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

impl<F> crate::Listener<F> for Listener<F>
where
    F: Framer + Clone + Unpin,
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
