use crate::{Connection, Framer};
use futures::Stream;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

pub trait Listener<F>: Stream<Item=Result<Box<dyn Connection<F, Error=crate::error::Error>>, crate::error::Error>>
where
    F: Framer + StdSend + 'static,
{
    fn connection_limit(&mut self, limit: usize);
}
