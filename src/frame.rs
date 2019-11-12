use bytes::BytesMut;
use crate::error::Error;

pub trait Framer {
    type Item;

    fn frame(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Error>;

    /// Return Ok(None) if need more bytes.
    fn deframe(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Error>;
}