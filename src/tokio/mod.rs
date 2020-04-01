//! Tokio-based TAPS implementation.
//!
//! This is an internal module and is not intended for direct use.

mod connection;
mod error;
mod listener;
mod race;

use crate::error::{box_error, Error};
use crate::implementation::Implementation;
use crate::{Endpoint, Framer, Listener};
pub use connection::Connection;

use crate::properties::TransportProperties;
use async_trait::async_trait;
use snafu::ResultExt;
use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Tokio;

#[async_trait]
impl Implementation for Tokio {
    async fn connection<F, L, R>(
        framer: F,
        _local: Option<L>,
        remote: R,
        props: TransportProperties,
    ) -> Result<Box<dyn crate::Connection<F>>, Error>
    where
        F: Framer,
        L: Endpoint,
        R: Endpoint,
    {
        race::race(remote, props, framer)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }

    async fn listener<F, L, R>(
        framer: F,
        local: L,
        remote: Option<R>,
        props: TransportProperties,
    ) -> Result<Box<dyn Listener<F, Item = Result<Box<dyn crate::Connection<F>>, Error>>>, Error>
    where
        F: Framer + Clone + Unpin,
        L: Endpoint,
        R: Endpoint,
    {
        let local_addr = local
            .resolve()
            .await
            .map_err(box_error)
            .with_context(|| crate::tokio::error::Resolve)
            .map_err(box_error)
            .map(|mut l| l.next().unwrap())
            .with_context(|| crate::error::Listen)?;
        let remote_addr = match remote {
            Some(r) => Some(
                r.resolve()
                    .await
                    .map_err(box_error)
                    .with_context(|| crate::tokio::error::Resolve)
                    .map_err(box_error)
                    .map(|mut l| l.next().unwrap())
                    .with_context(|| crate::error::Listen)?,
            ),
            None => None,
        };

        listener::open_listener(local_addr, remote_addr, props, framer)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Listen)
    }
}

#[async_trait]
impl<T> Endpoint for T
where
    T: ToSocketAddrs + Send + Sync + 'static,
{
    type Error = crate::resolve::Error;
    type Iter = impl Iterator<Item = SocketAddr>;

    async fn resolve(self) -> Result<Self::Iter, Self::Error> {
        ::tokio::net::lookup_host(self)
        .await
        .with_context(|| crate::resolve::Io)
    }
}
