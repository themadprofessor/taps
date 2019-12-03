use crate::error::{Error, box_error};
use crate::error::Initiate;
use crate::properties::TransportProperties;
use crate::tokio::connection::Connection;
use crate::tokio::error::NoEndpoint;
use crate::Endpoint;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt, Future};
use snafu::{OptionExt, ResultExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time;
use crate::frame::Framer;

fn add_delay<T, F>(
    addr: SocketAddr,
    props: &TransportProperties,
    framer: Option<F>
) -> impl Future<Output = Result<Box<dyn crate::Connection<T, F>>, Error>> + '_
where
    T: Send + 'static,
    F: Send + 'static + Framer
{
    match addr {
        SocketAddr::V4(_) => time::delay_for(Duration::from_millis(5)),
        SocketAddr::V6(_) => time::delay_for(Duration::from_nanos(0)),
    }
    .then(move |_| Connection::create(addr, props, framer))
}

pub async fn race<E, T, F>(
    endpoint: E,
    props: TransportProperties,
    framer: Option<F>
) -> Result<Box<dyn crate::Connection<T, F>>, Error>
where
    E: Endpoint + Send,
    T: Send + 'static,
    F: Send + 'static + Framer
{
    endpoint
        .resolve()
        .await?
        .into_iter()
        .map(|addr| add_delay(addr, &props, framer))
        .collect::<FuturesUnordered<_>>()
        .next()
        .await
        .with_context(|| NoEndpoint)
        .map_err(box_error)
        .with_context(|| Initiate)?
}
