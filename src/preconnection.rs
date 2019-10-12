use snafu::{Snafu, ResultExt};
use tokio_dns::{Endpoint, ToEndpoint};
use std::convert::TryFrom;
use std::marker::PhantomData;

/// A marker trait to specify types which represent the state of a
/// [Preconnection's](struct.Preconnection.html) endpoints.
pub trait EndpointState {}

/// Type used to represent a missing [LocalEndpoint](struct.LocalEndpoint.html) or
/// [RemoteEndpoint](struct.RemoteEndpoint.html) in a [Preconnection](struct.Preconnection.html).
pub struct NoEndpoint;
pub struct LocalEndpoint<'a, T>(T, &'a PhantomData<T>) where T: ToEndpoint<'a>;
pub struct RemoteEndpoint<'a, T>(T, &'a PhantomData<T>) where T: ToEndpoint<'a>;

/// A configuration type use to configure how to create a Connection.
pub struct Preconnection<L, R> where L: EndpointState, R: EndpointState {
    local: Box<L>,
    remote: Box<R>
}

impl EndpointState for NoEndpoint {}
impl <'a, T> EndpointState for LocalEndpoint<'a, T> where T: ToEndpoint<'a> {}
impl <'a, T> EndpointState for RemoteEndpoint<'a, T> where T: ToEndpoint<'a> {}

impl Preconnection<NoEndpoint, NoEndpoint> {
    /// Create a new Preconnection which has no endpoints specified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::taps::preconnection::*;
    /// let preconnection = Preconnection::new();
    /// ```
    pub fn new() -> Self {
        // Note: Box::new does not alloc when given 0-sized type.
        Preconnection {
            local: Box::new(NoEndpoint),
            remote : Box::new(NoEndpoint),
        }
    }
}

impl <L, R> Preconnection<L, R> where L: EndpointState, R: EndpointState {
    /// Specify the local endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    pub fn local_endpoint<'a, T>(self, local: T) -> Preconnection<LocalEndpoint<'a, T>, R> where T: ToEndpoint<'a> {
        Preconnection {
            local: Box::new(LocalEndpoint(local, &PhantomData)),
            remote: self.remote
        }
    }

    /// Specify the remote endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    ///
    /// No name resolution is done by this method. See resolve for eager resolution or initiate for
    /// late resolution.
    pub fn remote_endpoint<'a, T>(self, remote: T) -> Preconnection<L, RemoteEndpoint<'a, T>> where T: ToEndpoint<'a> {
        Preconnection {
            local: self.local,
            remote: Box::new(RemoteEndpoint(remote, &PhantomData)),
        }
    }
}