use super::Error;
use crate::{Decode, Encode};
use bytes::BytesMut;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

pub trait Framer<S, R>: Send + Sync + 'static {
    type MetaKey;
    type MetaValue;
    type Error: StdError + StdSend;

    fn frame(&mut self, item: S, dst: &mut BytesMut) -> Result<(), Self::Error>
    where
        S: Encode;

    fn deframe(&mut self, src: &mut BytesMut) -> Result<R, Error<Self::Error>>
    where
        R: Decode;

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue);
}
