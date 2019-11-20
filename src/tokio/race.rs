use crate::properties::TransportProperties;
use crate::tokio::error::{Connecting, Error, NoEndpoint, Resolution};
use futures::stream::FuturesUnordered;
use futures::FutureExt;
use futures::StreamExt;
use snafu::{OptionExt, ResultExt};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::future::Future;
use tokio::net::TcpStream;
use tokio::timer::{self};

use crate::Endpoint;
use crate::tokio::connection::stream::Stream;

fn add_delay(addr: SocketAddr) -> impl Future<Output = Result<TcpStream, ::std::io::Error>> {
    timer::delay_for(Duration::from_micros(if let SocketAddr::V4(_) = addr {
        5
    } else {
        0
    }))
    .then(move |_| TcpStream::connect(addr))
}

pub(crate) async fn race<T>(endpoint: T, props: &TransportProperties) -> Result<Stream<TcpStream>, Error>
where
    T: Endpoint,
{
    endpoint
        .resolve()
        .await
        .with_context(|| Resolution)?
        .into_iter()
        .map(add_delay)
        .collect::<FuturesUnordered<_>>()
        .next()
        .await
        .with_context(|| NoEndpoint)?
        .with_context(|| Connecting)
        .map(Stream::new)
}
