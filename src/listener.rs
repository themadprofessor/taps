use crate::error::Error;
use crate::{Connection, Framer};
use futures::Stream;

// The stream's item is a possible connection with the same framer and error types as this listener
pub trait Listener<F>: Stream<Item = Result<Box<dyn Connection<F>>, Error>>
where
    F: Framer,
{
    fn connection_limit(&mut self, limit: usize);
}
