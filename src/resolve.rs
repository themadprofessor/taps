use async_trait::async_trait;
use snafu::Snafu;
use std::error::Error as StdError;
use std::marker::Send as StdSend;
use std::net::SocketAddr;
use tokio::task;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
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
