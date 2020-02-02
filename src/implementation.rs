use crate::error::Error;
use crate::{Connection, Endpoint, Framer, Listener};
use async_trait::async_trait;
use futures::Stream;

#[async_trait]
pub trait Impl {
    async fn connection<F, L, R>(
        framer: F,
        local: Option<L>,
        remote: R,
    ) -> Result<Box<dyn Connection<F>>, Error>
    where
        F: Framer,
        L: Endpoint,
        R: Endpoint;

    async fn listener<F, L, R>(
        framer: F,
        local: L,
        remote: Option<R>,
    ) -> Result<Box<dyn Listener<F, Item = Result<Box<dyn Connection<F>>, Error>>>, Error>
    where
        F: Framer,
        L: Endpoint,
        R: Endpoint;
}
