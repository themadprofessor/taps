use crate::connection::Connection;
use crate::error::Error;
use crate::properties::TransportProperties;
use async_trait::async_trait;
use std::net::SocketAddr;

#[async_trait]
pub trait Endpoint {
    async fn resolve(self) -> Result<Vec<SocketAddr>, Error>;
}

#[async_trait]
pub trait Preconnection<T, L, R> {
    fn local_endpoint(&mut self, local: L)
    where
        L: Endpoint;

    fn remote_endpoint(&mut self, remote: R)
    where
        R: Endpoint;

    fn transport_properties(&self) -> &TransportProperties;

    fn transport_properties_mut(&mut self) -> &mut TransportProperties;

    async fn initiate(self) -> Result<Box<dyn Connection<T>>, Error>
    where
        T: Send + 'static;
}
