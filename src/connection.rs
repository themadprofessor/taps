use crate::frame::Framer;
use crate::{Decode, Encode};
use async_trait::async_trait;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

#[async_trait]
pub trait Connection<F: Framer + Send + 'static> {
    type Error: StdError + StdSend;

    /// Send data over this connection.
    async fn send(&mut self, data: F::Input) -> Result<(), Self::Error>
    where
        F::Input: Encode;

    /// Receive data from this connection.
    async fn receive(&mut self) -> Result<F::Output, Self::Error>
    where
        F::Output: Decode;

    /// Close this connection gracefully.
    async fn close(self: Box<Self>) -> Result<(), Self::Error>;

    /// Abort this connection ungracefully.
    fn abort(self: Box<Self>);
}
