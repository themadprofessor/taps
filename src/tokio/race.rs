use std::net::SocketAddr;
use std::time::Duration;

use futures::FutureExt;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use snafu::{OptionExt, ResultExt};
use tokio::future::Future;
use tokio::net::TcpStream;
use tokio::timer::{self};

use crate::Endpoint;
use crate::error::Connecting;
use crate::error::Error;
use crate::error::NoEndpoint;
use crate::properties::TransportProperties;

fn add_delay(addr: SocketAddr) -> impl Future<Output = Result<TcpStream, ::std::io::Error>> {
    timer::delay_for(Duration::from_micros(if let SocketAddr::V4(_) = addr {
        5
    } else {
        0
    }))
    .then(move |_| TcpStream::connect(addr))
}

pub(crate) async fn race<T>(
    endpoint: T,
    props: &TransportProperties,
) -> Result<TcpStream, Error>
where
    T: Endpoint,
{
    endpoint.resolve()
        .await?
        .into_iter()
        .map(add_delay)
        .collect::<FuturesUnordered<_>>()
        .next()
        .await
        .with_context(|| NoEndpoint)?
        .with_context(|| Connecting)
}
