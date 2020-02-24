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

pub struct Tokio;

#[async_trait]
impl Implementation for Tokio {
    async fn connection<F, L, R, SD, RD>(
        framer: F,
        _local: Option<L>,
        remote: R,
        props: &TransportProperties,
    ) -> Result<Box<dyn crate::Connection<F, SD, RD>>, Error>
    where
        F: Framer<SD, RD> + Unpin + Clone,
        L: Endpoint,
        R: Endpoint,
        SD: Send + Unpin + 'static,
        RD: Send + Unpin + 'static,
    {
        race::race(remote, props, framer)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }

    async fn listener<F, L, R, SD, RD>(
        framer: F,
        local: L,
        remote: Option<R>,
        props: &TransportProperties,
    ) -> Result<
        Box<dyn Listener<F, SD, RD, Item = Result<Box<dyn crate::Connection<F, SD, RD>>, Error>>>,
        Error,
    >
    where
        F: Framer<SD, RD>,
        L: Endpoint,
        R: Endpoint,
    {
        unimplemented!()
    }
}
