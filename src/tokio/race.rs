use crate::error::{Error, box_error};
use crate::error::Initiate;
use crate::properties::TransportProperties;
use crate::tokio::connection::Connection;
use crate::tokio::error::NoEndpoint;
use crate::Endpoint;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use snafu::{OptionExt, ResultExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::prelude::Future;
use tokio::timer;
use tokio::timer::Delay;

fn add_delay<T>(
    addr: SocketAddr,
    props: &TransportProperties,
) -> impl Future<Output = Result<Box<dyn crate::Connection<T>>, Error>> + '_
where
    T: Send + 'static,
{
    match addr {
        SocketAddr::V4(_) => timer::delay_for(Duration::from_millis(5)),
        SocketAddr::V6(_) => timer::delay_for(Duration::from_nanos(0)),
    }
    .then(move |_| Connection::create(addr, props))
}

pub async fn race<E, T>(
    endpoint: E,
    props: TransportProperties,
) -> Result<Box<dyn crate::Connection<T>>, Error>
where
    E: Endpoint + Send,
    T: Send + 'static,
{
    endpoint
        .resolve()
        .await?
        .into_iter()
        .map(|addr| add_delay(addr, &props))
        .collect::<FuturesUnordered<_>>()
        .next()
        .await
        .with_context(|| NoEndpoint)
        .map_err(box_error)
        .with_context(|| Initiate)?
}
