use crate::error::Error;
use crate::{Connection, Endpoint, Framer, Listener};
use async_trait::async_trait;
use crate::properties::TransportProperties;

#[async_trait]
pub trait Impl {
    async fn connection<F, L, R>(
        framer: F,
        local: Option<L>,
        remote: R,
        props: &TransportProperties
    ) -> Result<Box<dyn Connection<F>>, Error>
    where
        F: Framer + Clone,
        L: Endpoint,
        R: Endpoint;

    async fn listener<F, L, R>(
        framer: F,
        local: L,
        remote: Option<R>,
        props: &TransportProperties
    ) -> Result<Box<dyn Listener<F, Item = Result<Box<dyn Connection<F>>, Error>>>, Error>
    where
        F: Framer,
        L: Endpoint,
        R: Endpoint;
}
