use snafu::{Snafu, ResultExt};
use tokio_dns::{Endpoint, ToEndpoint};
use std::convert::TryFrom;
use std::marker::PhantomData;

pub trait EndpointState {}

pub struct NoEndpoint;
pub struct LocalEndpoint<'a, T>(T, &'a PhantomData<T>) where T: ToEndpoint<'a>;
pub struct RemoteEndpoint<'a, T>(T, &'a PhantomData<T>) where T: ToEndpoint<'a>;

pub struct Preconnection<L, R> where L: EndpointState, R: EndpointState {
    local: Box<L>,
    remote: Box<R>
}

impl EndpointState for NoEndpoint {}
impl <'a, T> EndpointState for LocalEndpoint<'a, T> where T: ToEndpoint<'a> {}
impl <'a, T> EndpointState for RemoteEndpoint<'a, T> where T: ToEndpoint<'a> {}

impl Preconnection<NoEndpoint, NoEndpoint> {
    /// Create a new Preconnection which has no endpoints specified.
    pub fn new() -> Self {
        // Note: Box::new does not alloc when given 0-sized type.
        Preconnection {
            local: Box::new(NoEndpoint),
            remote : Box::new(NoEndpoint),
        }
    }
}

impl <L, R> Preconnection<L, R> where L: EndpointState, R: EndpointState {
    ///
    pub fn local_endpoint<'a, T>(self, local: T) -> Preconnection<LocalEndpoint<'a, T>, R> where T: ToEndpoint<'a> {
        Preconnection {
            local: Box::new(LocalEndpoint(local, &PhantomData)),
            remote: self.remote
        }
    }

    pub fn remote_endpoint<'a, T>(self, remote: T) -> Preconnection<L, RemoteEndpoint<'a, T>> where T: ToEndpoint<'a> {
        Preconnection {
            local: self.local,
            remote: Box::new(RemoteEndpoint(remote, &PhantomData)),
        }
    }
}