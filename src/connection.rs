use crate::error::Error;
use crate::Framer;
use async_trait::async_trait;
use std::net::SocketAddr;

#[async_trait]
pub trait Connection<F>: Send
where
    F: Framer,
{
    /// Send data over this connection.
    async fn send(&mut self, data: F::Input) -> Result<(), Error>;

    /// Receive data from this connection.
    async fn receive(&mut self) -> Result<F::Output, Error>;

    /// Close this connection gracefully.
    async fn close(self: Box<Self>) -> Result<(), Error>;

    /// Abort this connection ungracefully.
    fn abort(self: Box<Self>);

    fn remote_endpoint(&self) -> SocketAddr;

    fn local_endpoint(&self) -> SocketAddr;
}
