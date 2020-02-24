use crate::error::Error;
use crate::properties::TransportProperties;
use crate::{Connection, Endpoint, Framer, Listener};
use async_trait::async_trait;

#[async_trait]
pub trait Implementation {
    async fn connection<F, L, R, SD, RD>(
        framer: F,
        local: Option<L>,
        remote: R,
        props: &TransportProperties,
    ) -> Result<Box<dyn Connection<F, SD, RD>>, Error>
    where
        F: Framer<SD, RD> + Clone,
        L: Endpoint,
        R: Endpoint,
        SD: Send + 'static,
        RD: Send + 'static;

    async fn listener<F, L, R, SD, RD>(
        framer: F,
        local: L,
        remote: Option<R>,
        props: &TransportProperties,
    ) -> Result<
        Box<dyn Listener<F, SD, RD, Item = Result<Box<dyn Connection<F, SD, RD>>, Error>>>,
        Error,
    >
    where
        F: Framer<SD, RD>,
        L: Endpoint,
        R: Endpoint;
}
