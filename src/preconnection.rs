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

use std::fmt;
use tokio_dns::ToEndpoint;
use crate::properties::TransportProperties;

/// A marker trait to specify types which represent the state of a
/// [Preconnection's](struct.Preconnection.html) endpoints.
pub trait EndpointState {}

/// Type used to represent a missing endpoint for a [Preconnection](struct.Preconnection.html).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NoEndpoint;

/// A configuration type used to configure how to create a Connection.
pub struct Preconnection<L, R>
where
    L: EndpointState,
    R: EndpointState,
{
    local: Box<L>,
    remote: Box<R>,
    trans_props: TransportProperties,
}

impl EndpointState for NoEndpoint {}
impl<'a, T> EndpointState for T where T: ToEndpoint<'a> {}

impl Preconnection<NoEndpoint, NoEndpoint> {
    /// Create a new Preconnection which has no endpoints specified.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use taps::preconnection::*;
    /// # use taps::properties::TransportProperties;
    /// let preconnection = Preconnection::new(TransportProperties::default());
    /// ```
    pub fn new(props: TransportProperties) -> Self {
        // Note: Box::new does not alloc when given 0-sized type.
        Preconnection {
            local: Box::new(NoEndpoint),
            remote: Box::new(NoEndpoint),
            trans_props: props
        }
    }
}

impl<L, R> fmt::Debug for Preconnection<L, R>
where
    L: EndpointState + fmt::Debug,
    R: EndpointState + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Preconnection")
            .field("local", &self.local)
            .field("remote", &self.remote)
            .finish()
    }
}

impl<L, R> Clone for Preconnection<L, R>
where
    L: EndpointState + Clone,
    R: EndpointState + Clone,
{
    fn clone(&self) -> Self {
        Preconnection {
            local: self.local.clone(),
            remote: self.remote.clone(),
            trans_props: self.trans_props.clone()
        }
    }
}

impl<L, R> PartialEq for Preconnection<L, R>
where
    L: EndpointState + PartialEq,
    R: EndpointState + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.local == other.local && self.remote == other.remote
    }
}

impl<L, R> Preconnection<L, R>
where
    L: EndpointState,
    R: EndpointState,
{
    /// Specify the local endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use taps::preconnection::*;
    /// # use taps::properties::TransportProperties;
    /// let preconnection = Preconnection::new(TransportProperties::default())
    ///     .local_endpoint("127.0.0.1:80");
    /// ```
    pub fn local_endpoint<'a, T>(self, local: T) -> Preconnection<T, R>
    where
        T: ToEndpoint<'a>,
    {
        Preconnection {
            local: Box::new(local),
            remote: self.remote,
            trans_props: self.trans_props
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
    /// # use taps::properties::TransportProperties;
    /// let preconnection = Preconnection::new(TransportProperties::default())
    ///     .remote_endpoint("example.com:80");
    /// ```
    pub fn remote_endpoint<'a, T>(self, remote: T) -> Preconnection<L, T>
    where
        T: ToEndpoint<'a>,
    {
        Preconnection {
            local: self.local,
            remote: Box::new(remote),
            trans_props: self.trans_props
        }
    }
}
