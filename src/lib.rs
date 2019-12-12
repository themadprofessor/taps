#![allow(dead_code)]
#![forbid(unsafe_code)]

use bytes::{Buf, Bytes, BytesMut};

pub use connection::Connection;
pub use frame::Framer;
pub use preconnection::*;

use crate::error::Error;
use crate::properties::TransportProperties;

mod connection;
pub mod error;
mod frame;
pub mod http;
mod preconnection;
pub mod properties;
mod tokio;

/// The `Encode` trait allows an object to be encoded.
///
///
///
/// # Implementation Example
/// ```
/// use taps::Encode;
/// use bytes::BytesMut;
/// use taps::error::Error;
///
/// struct MyVec(Vec<u8>);
///
/// impl Encode for MyVec {
///     fn encode(&self, data: &mut BytesMut) -> Result<(), Error> {
///         data.extend_from_slice(&self.0);
///         Ok(())
///     }
///     fn size_hint(&self) -> (usize, Option<usize>) {
///         (self.0.len(), Some(self.0.len()))
///     }
/// }
/// ```
///
/// An example of a failable implementation.
/// ```
/// use taps::Encode;
/// use bytes::BytesMut;
/// use taps::error::{Error, box_error};
/// use std::convert::TryInto;
/// use std::fmt;
/// use snafu::ResultExt;
/// use serde::export::Formatter;
///
/// struct MyFallible(Option<Vec<u8>>);
///
/// #[derive(Debug)]
/// struct EmptyOption;
///
/// impl fmt::Display for EmptyOption {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
///         f.write_str("empty option")
///     }
/// }
///
/// impl ::std::error::Error for EmptyOption {}
///
/// impl Encode for MyFallible {
///     fn encode(&self, data: &mut BytesMut) -> Result<(), Error> {
///         let vec = self.0.as_ref()
///             .ok_or_else(|| EmptyOption)
///             .map_err(box_error)
///             .with_context(|| ::taps::error::Encode)?;
///         data.extend_from_slice(&vec);
///         Ok(())
///     }
/// }
/// ```
pub trait Encode {
    /// Encode self into the given BytesMut.
    ///
    /// # Error
    /// Return `Ok(())` if the encoding was successful.
    ///
    /// Return `Err(error::Error::Encode)` if the encode failed.
    /// See example on how to produce this type.
    ///
    fn encode(&self, data: &mut BytesMut) -> Result<(), error::Error>;

    /// Returns the bounds of the expected encoded length of this object.
    ///
    /// Specifically, `size_hint()` returns a tuple where the first element is the lower bound, and
    /// the second element is the upper bound.
    ///
    /// The second element of the tuple is an `Option<usize>`. A `None` here means there is no upper
    /// bound, or the upper bound is larger than `usize`.
    ///
    /// # Implementation notes
    ///
    /// This is primarily used to reserve space in the `BytesMut` given to `encode`. Specifically,
    /// if the second element is `Some(val)`, then `val` bytes will be reserved. If the second
    /// element is `None`, then the first element is used as the number of bytes to reserve.
    ///
    /// Since this is primarily used for optimisations, the validity of the returned value must not
    /// be relied on to ensure safety. E.G. an invalid return value should not lead to memory safety
    /// violations.
    ///
    /// The default implementation is `(0, None)` which is always valid.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

/// The `Decode` trait allows an object to be decoded.
pub trait Decode {
    /// Attempt to decode an object from the given `Bytes.
    fn decode(data: &mut BytesMut) -> Result<Self, error::Error>
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
    fn decode(_data: &mut BytesMut) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Decode for Vec<u8> {
    fn decode(data: &mut BytesMut) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let res = data.to_vec();
        data.advance(res.len());
        Ok(res)
    }
}

impl Decode for String {
    fn decode(data: &mut BytesMut) -> Result<Self, Error>
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
