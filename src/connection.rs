use crate::error::Error;
use crate::{Decode, Encode};
use async_trait::async_trait;
use crate::frame::Framer;

#[async_trait]
pub trait Connection<T: Send + 'static, F: Framer + Send + 'static> {
    async fn send(&mut self, data: T) -> Result<(), Error>
    where
        T: Encode;
    async fn receive(&mut self) -> Result<T, Error>
    where
        T: Decode;
    async fn close(self: Box<Self>) -> Result<(), Error>;

    fn abort(self: Box<Self>);
}
