use async_trait::async_trait;
use std::error::Error as StdError;
use std::marker::Send as StdSend;
use std::net::SocketAddr;
use snafu::{Snafu, ResultExt};
use tokio::task;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("missing endpoint"))]
    MissingEndpoint,

    #[snafu(display("io error: {}", source))]
    Io {
        source: ::std::io::Error
    },

    #[snafu(display("failed to join resolver task: {}", source))]
    Join {
        source: task::JoinError
    }
}

/// The `Endpoint` trait allows resolving a domain name into `SocketAddr`s.
///
/// This trait does not have a blanket impl for `ToSocketAddr`, because that trait only resolves
/// into a single `SocketAddr`.
#[async_trait]
pub trait Endpoint {
    type Error: StdSend + StdError;

    /// Attempt to resolve this endpoint into a collection of SockerAddrs.
    async fn resolve(self) -> Result<Vec<SocketAddr>, Self::Error>;
}

#[async_trait]
impl Endpoint for SocketAddr {
    type Error = ::std::convert::Infallible;

    async fn resolve(self) -> Result<Vec<SocketAddr>, Self::Error> {
        Ok(vec![self])
    }
}

#[async_trait]
impl Endpoint for () {
    type Error = Error;

    #[allow(clippy::unit_arg)]
    async fn resolve(self) -> Result<Vec<SocketAddr>, Self::Error> {
        Err(Error::MissingEndpoint)
    }
}

#[async_trait]
impl<T> Endpoint for (T, u16)
where
    T: AsRef<str> + StdSend + 'static,
{
    type Error = Error;

    async fn resolve(self) -> Result<Vec<SocketAddr>, Self::Error> {
        task::spawn_blocking(move || {
            dns_lookup::lookup_host(self.0.as_ref()).map(|v| {
                v.into_iter()
                    .map(|ip| SocketAddr::from((ip, self.1)))
                    .collect::<Vec<_>>()
            })
        })
        .await
            .with_context(|| Join)?
            .with_context(|| Io)
    }
}
