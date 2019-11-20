use crate::properties::TransportProperties;
use crate::tokio::error::{Binding, Error, NoEndpoint, Resolution};
use crate::Endpoint;
use snafu::{OptionExt, ResultExt};
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::stream::Stream;
use tokio_dns::{resolve_sock_addr, ToEndpoint};

pub struct Listener<I> {
    incoming: I,
}

impl<I, C> Listener<I>
where
    I: Stream<Item = ::std::io::Result<C>>,
    C: AsyncRead + AsyncWrite,
{
    pub(crate) async fn create<'a, L>(
        endpoint: L,
        props: &TransportProperties,
    ) -> Result<Listener<::tokio::net::tcp::Incoming>, Error>
    where
        L: Endpoint,
    {
        let addr = endpoint
            .resolve()
            .await
            .with_context(|| Resolution)?
            .into_iter()
            .next()
            .with_context(|| NoEndpoint)?;

        Ok(Listener {
            incoming: TcpListener::bind(addr)
                .await
                .with_context(|| Binding)?
                .incoming(),
        })
    }
}

/*
impl<C, I> Stream for Listener<I>
where
    C: AsyncRead + AsyncWrite,
    I: Stream<Item = ::std::io::Result<C>>,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.incoming.poll_next(cx)
    }
}
*/
