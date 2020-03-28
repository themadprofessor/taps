use async_trait::async_trait;
use snafu::{ResultExt, Snafu};
use std::error::Error as StdError;
use std::marker::Send as StdSend;
use std::net::SocketAddr;
use tokio::task;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("missing endpoint"))]
    MissingEndpoint,

    #[snafu(display("io error: {}", source))]
    Io { source: ::std::io::Error },

    #[snafu(display("failed to join resolver task: {}", source))]
    Join { source: task::JoinError },
}

/// The `Endpoint` trait allows resolving a domain name into `SocketAddr`s.
///
/// This trait does not have a blanket impl for `ToSocketAddr`, because that trait only resolves
/// into a single `SocketAddr`.
#[async_trait]
pub trait Endpoint: Send + 'static {
    type Error: StdSend + StdError;
    type Iter: Iterator<Item = SocketAddr> + StdSend;

    /// Attempt to resolve this endpoint into a collection of SockerAddrs.
    async fn resolve(self) -> Result<Self::Iter, Self::Error>;
}

#[async_trait]
impl Endpoint for SocketAddr {
    type Error = ::std::convert::Infallible;
    type Iter = ::std::iter::Once<SocketAddr>;

    async fn resolve(self) -> Result<Self::Iter, Self::Error> {
        Ok(::std::iter::once(self))
    }
}

#[async_trait]
impl Endpoint for () {
    type Error = Error;
    type Iter = ::std::iter::Once<SocketAddr>;

    #[allow(clippy::unit_arg)]
    async fn resolve(self) -> Result<Self::Iter, Self::Error> {
        Err(Error::MissingEndpoint)
    }
}

#[async_trait]
impl<T> Endpoint for (T, u16)
where
    T: AsRef<str> + StdSend + Sync + 'static,
{
    type Error = Error;
    type Iter = impl Iterator<Item = SocketAddr>;

    async fn resolve(self) -> Result<Self::Iter, Self::Error> {
        task::spawn_blocking(move || {
            dns_lookup::lookup_host(self.0.as_ref())
                .map(move |v| v.into_iter().map(move |ip| SocketAddr::from((ip, self.1))))
        })
        .await
        .with_context(|| Join)?
        .with_context(|| Io)
    }
}
