use crate::error::Error;
use crate::{Decode, Encode};
use async_trait::async_trait;

#[async_trait]
pub trait Connection<T: Send + 'static> {
    async fn send(&mut self, data: T) -> Result<(), Error>
    where
        T: Encode;
    async fn receive(&mut self) -> Result<T, Error>
    where
        T: Decode;
    async fn close(self) -> Result<(), Error>;

    fn abort(self);
}
