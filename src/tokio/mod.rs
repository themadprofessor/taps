//! Tokio-based TAPS implementation.
//!
//! This is an internal module and is not intended for direct use.

mod connection;
mod error;
mod listener;
mod preconnection;
mod race;

use crate::error::Error;
use crate::implementation::Impl;
use crate::{Endpoint, Framer, Listener};
pub use connection::Connection;
pub use preconnection::Preconnection;

use async_trait::async_trait;
use futures::Stream;

pub struct Tokio;

#[async_trait]
impl Impl for Tokio {
    async fn connection<F, L, R>(
        framer: F,
        local: Option<L>,
        remote: R,
    ) -> Result<Box<dyn crate::Connection<F>>, Error>
    where
        F: Framer + Send + 'static,
        L: Endpoint,
        R: Endpoint,
    {
        unimplemented!()
    }

    async fn listener<F, L, R>(
        framer: F,
        local: L,
        remote: Option<R>,
    ) -> Result<Box<dyn Listener<F, Item = Result<Box<dyn crate::Connection<F>>, Error>>>, Error>
    where
        F: Framer + Send + 'static,
        L: Endpoint,
        R: Endpoint,
    {
        unimplemented!()
    }
}
