use crate::connection::Connection;
use crate::error::Error;
use crate::frame::Framer;
use crate::properties::TransportProperties;
use crate::resolve::Endpoint;

/// A marker trait to specify types which represent the state of a
/// [Preconnection's](struct.Preconnection.html) endpoints.
pub trait EndpointState {}

/// Type used to represent a missing endpoint for a [Preconnection](struct.Preconnection.html).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NoEndpoint;

/// A configuration type used to configure how to create a Connection and/or Listener.
///
/// # Generic Types
/// - F: The framer which the resulting Connection(s) will use to frame and deframe data
/// - L: The type of the local endpoint. It is [NoEndpoint](struct.NoEndpoint.html) if no local
/// endpoint is given
/// - R: The type of the remote endpoint. It is [NoEndpoint](struct.NoEndpoint.html) if no remote
/// endpoint is given
#[derive(Debug, Clone, PartialEq)]
pub struct Preconnection<F, L, R>
where
    F: Framer + Send + 'static,
    L: EndpointState,
    R: EndpointState,
{
    local: L,
    remote: R,
    framer: F,
    trans: TransportProperties,
}

impl EndpointState for NoEndpoint {}
impl<T> EndpointState for T where T: Endpoint {}

impl<F> Preconnection<F, NoEndpoint, NoEndpoint>
where
    F: Framer + Send + 'static,
{
    /// Create a new Preconnection which has no endpoints specified.
    pub fn new(props: TransportProperties, framer: F) -> Self {
        Preconnection {
            local: NoEndpoint,
            remote: NoEndpoint,
            trans: props,
            framer,
        }
    }
}

impl<F, L, R> Preconnection<F, L, R>
where
    L: EndpointState,
    R: EndpointState,
    F: Framer + Send + 'static,
{
    /// Specify the local endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    pub fn local_endpoint<N>(self, local: N) -> Preconnection<F, N, R>
    where
        N: Endpoint,
    {
        Preconnection {
            local,
            remote: self.remote,
            trans: self.trans,
            framer: self.framer,
        }
    }

    /// Specify the remote endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    ///
    /// No name resolution is done by this method. See resolve for eager resolution or initiate for
    /// late resolution.
    pub fn remote_endpoint<N>(self, remote: N) -> Preconnection<F, L, N>
    where
        N: Endpoint,
    {
        Preconnection {
            local: self.local,
            remote,
            trans: self.trans,
            framer: self.framer,
        }
    }

    pub fn transport_properties(&self) -> &TransportProperties {
        &self.trans
    }

    pub fn transport_properties_mut(&mut self) -> &mut TransportProperties {
        &mut self.trans
    }
}

impl<F, L, R> Preconnection<F, L, R>
where
    L: EndpointState,
    R: Endpoint,
    F: Framer + Send + 'static,
{
    pub async fn initiate(self) -> Result<Box<dyn Connection<F>>, Error> {
        unimplemented!()
    }
}
