use crate::frame::Framer;
use crate::{Decode, Encode};
use async_trait::async_trait;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

#[async_trait]
pub trait Connection<F: Framer + Send + 'static> {
    type Error: StdError + StdSend;

    async fn send(&mut self, data: F::Input) -> Result<(), Self::Error>
    where
        F::Input: Encode;
    async fn receive(&mut self) -> Result<F::Output, Self::Error>
    where
        F::Output: Decode;
    async fn close(self: Box<Self>) -> Result<(), Self::Error>;

    fn abort(self: Box<Self>);
}
