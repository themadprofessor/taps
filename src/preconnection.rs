//! A [Preconnection](struct.Preconnection.html) represents a potential connection.
//!
//! It has state that describes the properties of a Connection that may exist in the future.
//! This state consists of Local Endpoint, Remote Endpoint, Transport Properties and security
//! parameters.
//!
//! The [Preconnection](struct.Preconnection.html)'s L and R type parameters are used to determine
//! how a [Preconnection](struct.Preconnection.html) can be used to create Connections or Listeners.
//! The L and R type parameters represent the Local Endpoint and Remote Endpoint respectively.
//!
//! If a [Preconnection](struct.Preconnection.html) has a [NoEndpoint](struct.NoEndpoint.html) as
//! its L type parameter, the [Preconnection](struct.Preconnection.html) cannot be used to create a
//! Listener via the listen method.
//!
//! If a [Preconnection](struct.Preconnection.html) has a [NoEndpoint](struct.NoEndpoint.html) as
//! its R type parameter, the [Preconnection](struct.Preconnection.html) cannot be used to create a
//! Connection via the initiate method.
//!
//! If a [Preconnection](struct.Preconnection.html) has a NoEndpoint as both its L and R type
//! parameters, the [Preconnection](struct.Preconnection.html) cannot be used to create a Connection
//! via the rendezvous method.

use snafu::{Snafu, ResultExt};
use tokio_dns::{Endpoint, ToEndpoint};
use std::convert::TryFrom;
use std::marker::PhantomData;

/// A marker trait to specify types which represent the state of a
/// [Preconnection's](struct.Preconnection.html) endpoints.
pub trait EndpointState {}

/// Type used to represent a missing endpoint for a [Preconnection](struct.Preconnection.html).
pub struct NoEndpoint;

/// A configuration type use to configure how to create a Connection.
pub struct Preconnection<L, R> where L: EndpointState, R: EndpointState {
    local: Box<L>,
    remote: Box<R>
}

impl EndpointState for NoEndpoint {}
impl <'a, T> EndpointState for T where T: ToEndpoint<'a> {}

impl Preconnection<NoEndpoint, NoEndpoint> {
    /// Create a new Preconnection which has no endpoints specified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use taps::preconnection::*;
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use taps::preconnection::*;
    /// let preconnection = Preconnection::new()
    ///     .local_endpoint("127.0.0.1:80");
    /// ```
    pub fn local_endpoint<'a, T>(self, local: T) -> Preconnection<T, R> where T: ToEndpoint<'a> {
        Preconnection {
            local: Box::new(local),
            remote: self.remote
        }
    }

    /// Specify the remote endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    ///
    /// No name resolution is done by this method. See resolve for eager resolution or initiate for
    /// late resolution.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use taps::preconnection::*;
    /// let preconnection = Preconnection::new()
    ///     .remote_endpoint("example.com:80");
    /// ```
    pub fn remote_endpoint<'a, T>(self, remote: T) -> Preconnection<L, T> where T: ToEndpoint<'a> {
        Preconnection {
            local: self.local,
            remote: Box::new(remote),
        }
    }
}