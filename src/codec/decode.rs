use bytes::{Buf, BytesMut};

use super::Error;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

/// The `Decode` trait allows an object to be decoded.
pub trait Decode {
    type Error: StdSend + StdError;

    /// Attempt to decode an object from the given `Bytes.
    fn decode(data: &mut BytesMut) -> Result<Self, Error<Self::Error>>
    where
        Self: Sized;
}

impl Decode for () {
    type Error = ::std::convert::Infallible;

    fn decode(_data: &mut BytesMut) -> Result<Self, Error<Self::Error>>
    where
        Self: Sized,
    {
        Ok(())
    }
}
