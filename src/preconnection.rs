use async_trait::async_trait;
use std::net::SocketAddr;
use crate::error::Error;
use crate::connection::Connection;

#[async_trait]
pub trait Endpoint {
    async fn resolve(self) -> Result<Vec<SocketAddr>, Error>;
}

#[async_trait]
pub trait Preconnection<T> {
    fn local_endpoint<N>(&mut self, local: N) where N: Endpoint;
    fn remote_endpoint<N>(&mut self, remote: N) where N: Endpoint;
    async fn initiate<C>(self) -> Result<C, Error> where C: Connection<T>, T: Send + 'static;
}
