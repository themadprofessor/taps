#![allow(dead_code)]

use bytes::{Bytes, BytesMut};

mod connection;
pub mod error;
mod frame;
mod preconnection;
pub mod properties;
mod tokio;
pub mod http;

use crate::error::Error;
use crate::properties::TransportProperties;
pub use connection::Connection;
pub use preconnection::*;
use crate::frame::Framer;

pub trait Encode {
    fn encode(&self, data: &mut BytesMut) -> Result<(), error::Error>;
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait Decode {
    fn decode(data: &Bytes) -> Result<Self, error::Error>
    where
        Self: Sized;
}

impl<T> Encode for T
where
    T: AsRef<[u8]>,
{
    fn encode(&self, data: &mut BytesMut) -> Result<(), Error> {
        data.extend_from_slice(self.as_ref());
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.as_ref().len(), Some(self.as_ref().len()))
    }
}

impl Decode for () {
    fn decode(data: &Bytes) -> Result<Self, Error> where
        Self: Sized {
        Ok(())
    }
}

impl Decode for Vec<u8> {
    fn decode(data: &Bytes) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(data.to_vec())
    }
}

pub fn new_preconnection<T, L, R, F>(props: TransportProperties) -> impl Preconnection<T, L, R, F>
where
    L: Endpoint + Send,
    R: Endpoint + Send,
    F: Framer + Send + 'static,
{
    crate::tokio::preconnection::Preconnection::new(props)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
