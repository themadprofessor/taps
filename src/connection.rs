use crate::error::Error;
use crate::frame::Framer;
use crate::{Decode, Encode};
use async_trait::async_trait;

#[async_trait]
pub trait Connection<F: Framer> {
    /// Send data over this connection.
    async fn send(&mut self, data: F::Input) -> Result<(), Error>
    where
        F::Input: Encode;

    /// Receive data from this connection.
    async fn receive(&mut self) -> Result<F::Output, Error>
    where
        F::Output: Decode;

    /// Close this connection gracefully.
    async fn close(self: Box<Self>) -> Result<(), Error>;

    /// Abort this connection ungracefully.
    fn abort(self: Box<Self>);
}
