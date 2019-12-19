use crate::connection::Connection;
use crate::frame::Framer;
use crate::properties::TransportProperties;
use crate::resolve::Endpoint;
use async_trait::async_trait;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

#[async_trait]
pub trait Preconnection<L, R, F> {
    type Error: StdSend + StdError;

    fn local_endpoint(&mut self, local: L)
    where
        L: Endpoint;

    fn remote_endpoint(&mut self, remote: R)
    where
        R: Endpoint;

    fn transport_properties(&self) -> &TransportProperties;

    fn transport_properties_mut(&mut self) -> &mut TransportProperties;

    fn add_framer(&mut self, framer: F);

    async fn initiate(self) -> Result<Box<dyn Connection<F, Error = Self::Error>>, Self::Error>
    where
        R: Endpoint + Send,
        <R as Endpoint>::Error: 'static,
        F: Framer + Send,
        F::Input: ::std::marker::Send;
}
