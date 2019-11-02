use crate::connection::Connection;
use crate::error::Connecting;
use crate::error::Error;
use crate::error::Resolution;
use crate::properties::TransportProperties;
use futures::compat::Future01CompatExt;
use futures::future::Then;
use futures::stream::FuturesUnordered;
use futures::{future::select_all, FutureExt, StreamExt, TryFutureExt};
use snafu::ResultExt;
use std::cmp::Ordering;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::future::Future;
use tokio::net::TcpStream;
use tokio::timer::{self, Delay};
use tokio_dns::ToEndpoint;

fn order_addrs(l: &SocketAddr, r: &SocketAddr) -> Ordering {
    match l {
        SocketAddr::V6(_) => match r {
            SocketAddr::V4(_) => Ordering::Greater,
            SocketAddr::V6(_) => Ordering::Equal,
        },
        SocketAddr::V4(_) => match r {
            SocketAddr::V4(_) => Ordering::Equal,
            SocketAddr::V6(_) => Ordering::Less,
        },
    }
}

async fn resolve<'a, T>(endpoint: T) -> Result<Vec<SocketAddr>, Error>
where
    T: ToEndpoint<'a>,
{
    let mut addrs = tokio_dns::resolve_sock_addr(endpoint)
        .compat()
        .await
        .with_context(|| Resolution)?;
    // Prioritise V6
    addrs.sort_unstable_by(order_addrs);

    Ok(addrs)
}

fn add_delay(addr: SocketAddr) -> impl Future<Output=Result<TcpStream, ::std::io::Error>> {
    timer::delay_for(Duration::from_micros(if let SocketAddr::V4(_) = addr {
        5
    } else {
        0
    }))
    .then(move |_| TcpStream::connect(addr))
}

pub(crate) async fn race<'a, T>(
    endpoint: T,
    props: &TransportProperties,
) -> Result<TcpStream, Error>
where
    T: ToEndpoint<'a>,
{
    resolve(endpoint)
        .await?
        .into_iter()
        .map(add_delay)
        .collect::<FuturesUnordered<_>>()
        .next()
        .await
        .unwrap()
        .with_context(|| Connecting)
}
