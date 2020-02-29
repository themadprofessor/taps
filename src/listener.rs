use crate::error::Error;
use crate::{Connection, Framer};
use futures::Stream;
use std::net::SocketAddr;

// The stream's item is a possible connection with the same framer and error types as this listener
pub trait Listener<F, S, R>: Stream<Item = Result<Box<dyn Connection<F, S, R>>, Error>> + Unpin
where
    F: Framer<S, R>,
    S: Unpin,
    R: Unpin
{
    fn connection_limit(&mut self, limit: usize);

    fn local_endpoint(&self) -> SocketAddr;

    fn remote_endpoint(&self) -> SocketAddr;
}
