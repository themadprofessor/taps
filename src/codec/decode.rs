use bytes::BytesMut;

use super::DecodeError;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

/// The `Decode` trait allows an object to be decoded.
pub trait Decode {
    type Error: StdSend + StdError;
    type State: Default;

    /// Attempt to decode an object from the given `Bytes.
    fn decode(
        data: &mut BytesMut,
        state: Self::State,
    ) -> Result<Self, DecodeError<Self::Error, Self::State>>
    where
        Self: Sized;
}

impl Decode for () {
    type Error = ::std::convert::Infallible;
    type State = ();

    fn decode(
        _data: &mut BytesMut,
        _state: (),
    ) -> Result<Self, DecodeError<Self::Error, Self::State>>
    where
        Self: Sized,
    {
        Ok(())
    }
}
