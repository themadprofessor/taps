use crate::{Connection, Framer};
use futures::Stream;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

// The stream's item is a possible connection with the same framer and error types as this listener
pub trait Listener<F>:
    Stream<
    Item = Result<
        Box<dyn Connection<F, Error = <Self as Listener<F>>::Error> + Send>,
        <Self as Listener<F>>::Error,
    >,
>
where
    F: Framer + StdSend + 'static,
{
    type Error: StdError + StdSend;

    fn connection_limit(&mut self, limit: usize);
}
