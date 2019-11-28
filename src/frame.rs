use crate::error::Error;
use bytes::BytesMut;

pub trait Framer {
    type Input;
    type Output;
    type MetaKey;
    type MetaValue;

    fn frame(&mut self, item: Self::Input, dst: &mut BytesMut) -> Result<(), Error>;

    /// Return Ok(None) if need more bytes.
    fn deframe(&mut self, src: &mut BytesMut) -> Result<Option<Self::Output>, Error>;

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue);
}
