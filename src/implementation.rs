use crate::error::Error;
use crate::properties::TransportProperties;
use crate::{Connection, Endpoint, Framer, Listener};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Implementation {
    async fn connection<F, L, R>(
        framer: F,
        local: Option<L>,
        remote: R,
        props: Arc<TransportProperties>,
    ) -> Result<Box<dyn Connection<F>>, Error>
    where
        F: Framer,
        L: Endpoint,
        R: Endpoint;

    async fn listener<F, L, R>(
        framer: F,
        local: L,
        remote: Option<R>,
        props: Arc<TransportProperties>,
    ) -> Result<Box<dyn Listener<F, Item = Result<Box<dyn Connection<F>>, Error>>>, Error>
    where
        F: Framer + Clone + Unpin,
        L: Endpoint,
        R: Endpoint;
}
