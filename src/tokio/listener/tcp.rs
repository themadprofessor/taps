use crate::error::box_error;
use crate::tokio::error::Error;
use crate::tokio::error::Listen;
use crate::tokio::Connection as TokioConnection;
use crate::{Connection, Framer, MakeSimilar};
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
    remote: Option<SocketAddr>,
}

impl<F> Listener<F> {
    pub(crate) async fn create(
        local: SocketAddr,
        remote: Option<SocketAddr>,
        framer: F,
    ) -> Result<Box<dyn crate::Listener<F>>, Error>
    where
        F: Framer + MakeSimilar + Unpin,
    {
        let inner = TcpListener::bind(local).await.with_context(|| Listen)?;
        Ok(Box::new(Listener {
            limit: None,
            framer,
            local,
            inner,
            remote,
        }))
    }
}

impl<F> Stream for Listener<F>
where
    F: Framer + MakeSimilar + Unpin,
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
                            TokioConnection::from_existing(raw, self.framer.make_similar(), remote)
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
    F: Framer + MakeSimilar + Unpin,
{
    fn connection_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    fn local_endpoint(&self) -> SocketAddr {
        self.local
    }

    fn remote_endpoint(&self) -> Option<SocketAddr> {
        self.remote
    }
}
