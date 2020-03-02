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
    async fn connection<F, L, R>(
        framer: F,
        _local: Option<L>,
        remote: R,
        props: &TransportProperties,
    ) -> Result<Box<dyn crate::Connection<F>>, Error>
    where
        F: Framer + Clone,
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
        props: &TransportProperties,
    ) -> Result<Box<dyn Listener<F, Item = Result<Box<dyn crate::Connection<F>>, Error>>>, Error>
    where
        F: Framer,
        L: Endpoint,
        R: Endpoint,
    {
        unimplemented!()
    }
}
