use bytes::BytesMut;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

pub trait Framer {
    type Input;
    type Output;
    type MetaKey;
    type MetaValue;
    type Error: StdError + StdSend;

    fn frame(&mut self, item: Self::Input, dst: &mut BytesMut) -> Result<(), Self::Error>;

    /// Return Ok(None) if need more bytes.
    fn deframe(&mut self, src: &mut BytesMut) -> Result<Option<Self::Output>, Self::Error>;

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue);
}
