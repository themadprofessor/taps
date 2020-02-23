use crate::error::Error;
use crate::Framer;
use crate::{Decode, Encode};
use async_trait::async_trait;
use std::net::SocketAddr;

#[async_trait]
pub trait Connection<F, S, R>: Send
where
    F: Framer<S, R>,
{
    /// Send data over this connection.
    async fn send(&mut self, data: S) -> Result<(), Error>;

    /// Receive data from this connection.
    async fn receive(&mut self) -> Result<R, Error>;

    /// Close this connection gracefully.
    async fn close(self: Box<Self>) -> Result<(), Error>;

    /// Abort this connection ungracefully.
    fn abort(self: Box<Self>);

    fn remote_endpoint(&self) -> SocketAddr;

    fn local_endpoint(&self) -> SocketAddr;
}
