use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Error<E> {
    Err(E),
    Incomplete,
}

impl<E> StdError for Error<E> where E: StdError {}

impl<E> fmt::Display for Error<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Err(e) => write!(f, "{}", e),
            Error::Incomplete => f.write_str("more data needed"),
        }
    }
}
