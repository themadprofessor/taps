use crate::error::Error;
use crate::frame::Framer;
use crate::{Decode, Encode};
use async_trait::async_trait;

#[async_trait]
pub trait Connection<F: Framer + Send + 'static> {
    async fn send(&mut self, data: F::Input) -> Result<(), Error>
    where
        F::Input: Encode;
    async fn receive(&mut self) -> Result<F::Output, Error>
    where
        F::Output: Decode;
    async fn close(self: Box<Self>) -> Result<(), Error>;

    fn abort(self: Box<Self>);
}
