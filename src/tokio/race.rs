use crate::error::Initiate;
use crate::error::{box_error, Error};
use crate::frame::Framer;
use crate::properties::TransportProperties;
use crate::tokio::connection::Connection;
use crate::tokio::error::NoEndpoint;
use crate::Endpoint;
use futures::stream::FuturesUnordered;
use futures::{Future, FutureExt, StreamExt};
use snafu::{OptionExt, ResultExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time;

fn add_delay<F>(
    addr: SocketAddr,
    props: &TransportProperties,
    framer: Option<F>,
) -> impl Future<Output = Result<Box<dyn crate::Connection<F>>, Error>> + '_
where
    F: Send + 'static + Framer,
    F::Input: ::std::marker::Send,
{
    match addr {
        SocketAddr::V4(_) => time::delay_for(Duration::from_millis(5)),
        SocketAddr::V6(_) => time::delay_for(Duration::from_nanos(0)),
    }
    .then(move |_| Connection::create(addr, props, framer))
}

pub async fn race<E, F>(
    endpoint: E,
    props: TransportProperties,
    framer: Option<F>,
) -> Result<Box<dyn crate::Connection<F>>, Error>
where
    E: Endpoint + Send,
    F: Send + 'static + Framer + Clone,
    F::Input: ::std::marker::Send,
{
    endpoint
        .resolve()
        .await?
        .into_iter()
        .map(|addr| add_delay(addr, &props, framer.clone()))
        .collect::<FuturesUnordered<_>>()
        .next()
        .await
        .with_context(|| NoEndpoint)
        .map_err(box_error)
        .with_context(|| Initiate)?
}
