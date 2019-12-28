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

    /// Specify the local endpoint for this preconnection.
    fn local_endpoint(&mut self, local: L)
    where
        L: Endpoint;

    /// Specify the remote endpoint for this preconnection.
    fn remote_endpoint(&mut self, remote: R)
    where
        R: Endpoint;

    /// Get this preconnection's transport properties.
    fn transport_properties(&self) -> &TransportProperties;

    /// Get a mutable reference to this preconnection's transport properties.
    fn transport_properties_mut(&mut self) -> &mut TransportProperties;

    /// Add a framer to this preconnection.
    fn add_framer(&mut self, framer: F);

    /// Attempt to initiate a connection from this preconnection.
    async fn initiate(self) -> Result<Box<dyn Connection<F, Error = Self::Error> + Send>, Self::Error>
    where
        R: Endpoint + Send,
        <R as Endpoint>::Error: 'static,
        F: Framer + Send,
        F::Input: ::std::marker::Send;
}
