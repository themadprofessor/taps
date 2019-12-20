use crate::error::box_error;
use crate::frame::Framer;
use crate::properties::TransportProperties;
use crate::tokio::connection::Connection;
use crate::tokio::error::{Error, Resolve};
use crate::Endpoint;
use futures::stream::FuturesUnordered;
use futures::{Future, FutureExt, StreamExt};
use snafu::{OptionExt, ResultExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time;
use log::{debug, trace};

fn add_delay<F>(
    addr: SocketAddr,
    props: &TransportProperties,
    framer: F,
) -> impl Future<Output = Result<Box<dyn crate::Connection<F, Error = Error> + Send>, Error>> + '_
where
    F: Send + 'static + Framer,
    F::Input: ::std::marker::Send,
{
    match addr {
        SocketAddr::V4(_) => {
            trace!("delaying v4");
            time::delay_for(Duration::from_millis(5))
        },
        SocketAddr::V6(_) => time::delay_for(Duration::from_millis(0)),
    }
    .then(move |_| Connection::create(addr, props, framer))
}

pub async fn race<E, F>(
    endpoint: E,
    props: TransportProperties,
    framer: F,
) -> Result<Box<dyn crate::Connection<F, Error = Error> + Send>, Error>
where
    E: Endpoint + Send,
    <E as Endpoint>::Error: 'static,
    F: Send + 'static + Framer + Clone,
    F::Input: ::std::marker::Send,
{
    debug!("racing");
    endpoint
        .resolve()
        .await
        .map_err(box_error)
        .with_context(|| Resolve)?
        .into_iter()
        .map(|addr| add_delay(addr, &props, framer.clone()))
        .collect::<FuturesUnordered<_>>()
        .fold(Err(Error::NoEndpoint), |acc, res| {
            ::futures::future::ready(if acc.is_ok() {
                acc
            } else {
                res
            })
        })
        .await
}
