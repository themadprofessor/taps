use async_trait::async_trait;
use bytes::{Bytes, BytesMut};

pub mod error;
pub mod properties;
mod connection;
mod preconnection;
mod tokio;

pub use connection::Connection;
pub use preconnection::*;
use crate::properties::TransportProperties;

#[async_trait]
pub trait Encode {
    async fn encode<T>(&self, data: BytesMut) -> Result<(), error::Error>;
}

pub trait Decode {
    fn decode(data: Bytes) -> Result<Self, error::Error> where Self: Sized;
}

pub fn new_preconnection<T>(props: TransportProperties) -> impl Preconnection<T> {
    tokio::Preconnection::new(props)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
