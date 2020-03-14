use crate::error::Error;
use crate::properties::TransportProperties;
use crate::{Connection, Endpoint, Framer, Listener, MakeSimilar};
use async_trait::async_trait;

#[async_trait]
pub trait Implementation {
    async fn connection<F, L, R>(
        framer: F,
        local: Option<L>,
        remote: R,
        props: &TransportProperties,
    ) -> Result<Box<dyn Connection<F>>, Error>
    where
        F: Framer,
        L: Endpoint,
        R: Endpoint;

    async fn listener<F, L, R>(
        framer: F,
        local: L,
        remote: Option<R>,
        props: &TransportProperties,
    ) -> Result<Box<dyn Listener<F, Item = Result<Box<dyn Connection<F>>, Error>>>, Error>
    where
        F: Framer + MakeSimilar + Unpin,
        L: Endpoint,
        R: Endpoint;
}
