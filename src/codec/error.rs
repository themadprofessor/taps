use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DecodeError<E, S> {
    Err(E),
    Incomplete(S),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeframeError<E> {
    Err(E),
    Incomplete,
}

impl<E, S> StdError for DecodeError<E, S>
where
    E: StdError,
    S: fmt::Display + fmt::Debug,
{
}
impl<E> StdError for DeframeError<E> where E: StdError {}

impl<E, S> fmt::Display for DecodeError<E, S>
where
    E: fmt::Display,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Err(e) => write!(f, "{}", e),
            DecodeError::Incomplete(s) => write!(f, "more data needed {}", s),
        }
    }
}

impl<E> fmt::Display for DeframeError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeframeError::Err(e) => write!(f, "{}", e),
            DeframeError::Incomplete => write!(f, "more data needed"),
        }
    }
}
