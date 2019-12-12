use crate::connection::Connection;
use crate::error::Error;
use crate::frame::Framer;
use crate::properties::TransportProperties;
use async_trait::async_trait;
use futures::StreamExt;
use std::net::SocketAddr;

/// The `Endpoint` trait allows resolving a domain name into `SocketAddr`s.
#[async_trait]
pub trait Endpoint {
    async fn resolve(self) -> Result<Vec<SocketAddr>, Error>;
}

#[async_trait]
impl<T> Endpoint for T
where
    T: Into<SocketAddr> + Send,
{
    async fn resolve(self) -> Result<Vec<SocketAddr>, crate::Error> {
        Ok(vec![self.into()])
    }
}

#[async_trait]
pub trait Preconnection<L, R, F> {
    fn local_endpoint(&mut self, local: L)
    where
        L: Endpoint;

    fn remote_endpoint(&mut self, remote: R)
    where
        R: Endpoint;

    fn transport_properties(&self) -> &TransportProperties;

    fn transport_properties_mut(&mut self) -> &mut TransportProperties;

    fn add_framer(&mut self, framer: F);

    async fn initiate(self) -> Result<Box<dyn Connection<F>>, Error>
    where
        R: Endpoint + Send,
        F: Framer + Send,
        F::Input: ::std::marker::Send;
}
