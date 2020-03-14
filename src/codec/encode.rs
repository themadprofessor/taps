use bytes::BytesMut;

use std::error::Error as StdError;
use std::marker::Send as StdSend;

/// The `Encode` trait allows an object to be encoded.
///
/// # Implementation Example
/// ```
/// use taps::Encode;
/// use bytes::BytesMut;
///
/// struct MyVec(Vec<u8>);
///
/// impl Encode for MyVec {
///     type Error = ::std::convert::Infallible;
///
///     fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
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
/// use std::fmt;
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
///     type Error = EmptyOption;
///
///     fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
///         let vec = self.0.as_ref()
///             .ok_or_else(|| EmptyOption)?;
///         data.extend_from_slice(&vec);
///         Ok(())
///     }
/// }
/// ```
pub trait Encode {
    type Error: StdSend + StdError + 'static;

    /// Encode self into the given BytesMut.
    ///
    /// # Error
    /// Return `Ok(())` if the encoding was successful.
    ///
    /// Return `Err(Self::Error)` if the encode failed.
    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error>;

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

impl Encode for &[u8] {
    type Error = ::std::convert::Infallible;

    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
        data.extend_from_slice(self);
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl Encode for &str {
    type Error = ::std::convert::Infallible;

    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
        data.extend_from_slice(self.as_bytes());
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl Encode for String {
    type Error = ::std::convert::Infallible;

    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
        data.extend_from_slice(self.as_ref());
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl Encode for () {
    type Error = ::std::convert::Infallible;

    fn encode(&self, _data: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}
