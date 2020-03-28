use std::net::SocketAddr;
use std::time::Duration;

use futures::FutureExt;
use log::{debug, trace};
use snafu::ResultExt;
use tokio::time;

use crate::error::box_error;
use crate::properties::TransportProperties;
use crate::tokio::connection::Connecting;
use crate::tokio::error::{Error, Resolve};
use crate::Endpoint;
use crate::Framer;

async fn add_delay(addr: SocketAddr, props: &TransportProperties) -> Result<Connecting, Error> {
    match addr {
        SocketAddr::V4(_) => {
            trace!("delaying v4");
            time::delay_for(Duration::from_millis(5)).await
        }
        SocketAddr::V6(_) => time::delay_for(Duration::from_millis(0)).await,
    };
    Connecting::create(addr, props).await
}

pub async fn race<E, F>(
    endpoint: E,
    props: &TransportProperties,
    framer: F,
) -> Result<Box<dyn crate::Connection<F>>, Error>
where
    E: Endpoint,
    <E as Endpoint>::Error: 'static,
    F: Framer,
{
    debug!("racing");
    ::futures::future::select_ok(
        endpoint
            .resolve()
            .await
            .map_err(box_error)
            .with_context(|| Resolve)?
            .map(|addr| add_delay(addr, &props).boxed()),
    )
    .await
    .map(|x| x.0.framer(framer))
}
