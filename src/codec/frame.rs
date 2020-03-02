use super::Error;
use crate::{Decode, Encode};
use bytes::BytesMut;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

pub trait Framer: Send + Sync + 'static {
    type Input: Encode + Send;
    type Output: Decode;
    type MetaKey;
    type MetaValue;
    type Error: StdError + StdSend + 'static;

    fn frame(&mut self, item: Self::Input, dst: &mut BytesMut) -> Result<(), Self::Error>;

    fn deframe(&mut self, src: &mut BytesMut) -> Result<Self::Output, Error<Self::Error>>;

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue);
}
