#![allow(dead_code)]
#![forbid(unsafe_code)]

use bytes::{Bytes, BytesMut};

mod connection;
pub mod error;
mod frame;
pub mod http;
mod preconnection;
pub mod properties;
mod tokio;

use crate::error::Error;
use crate::frame::Framer;
use crate::properties::TransportProperties;
pub use connection::Connection;
pub use preconnection::*;

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

impl Encode for &[u8] {
    fn encode(&self, data: &mut BytesMut) -> Result<(), Error> {
        data.extend_from_slice(self);
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl Encode for &str {
    fn encode(&self, data: &mut BytesMut) -> Result<(), Error> {
        data.extend_from_slice(self.as_bytes());
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl Encode for String {
    fn encode(&self, data: &mut BytesMut) -> Result<(), Error> {
        data.extend_from_slice(self.as_ref());
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl Decode for () {
    fn decode(_data: &Bytes) -> Result<Self, Error>
    where
        Self: Sized,
    {
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

impl Decode for String {
    fn decode(data: &Bytes) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(String::from_utf8_lossy(data).to_string())
    }
}

pub fn new_preconnection<T, L, R, F>(props: TransportProperties) -> impl Preconnection<L, R, F>
where
    L: Endpoint + Send,
    R: Endpoint + Send,
    F: Framer + Send + Sync + Clone + 'static,
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
