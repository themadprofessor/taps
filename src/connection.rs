use async_trait::async_trait;
use crate::{Encode, Decode};
use crate::error::Error;

#[async_trait]
pub trait Connection<T: Send + 'static> {
    async fn send(&mut self, data: T) where T: Encode;
    async fn receive(&mut self) where T: Decode;
    fn close(self) -> Result<(), Error>;
}