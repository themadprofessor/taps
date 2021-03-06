use crate::connection::Connection;
use crate::error::{box_error, Error};
use crate::implementation::Implementation;
use crate::properties::TransportProperties;
use crate::resolve::Endpoint;
use crate::Framer;
use crate::Listener;
use snafu::ResultExt;
use std::marker::PhantomData;

#[cfg(feature = "tokio-impl")]
pub type DefaultImpl = crate::tokio::Tokio;

#[cfg(not(feature = "tokio-impl"))]
compile_error!("no implementation feature enabled");

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
pub struct Preconnection<F, L, R, I = DefaultImpl>
where
    F: Framer,
    L: EndpointState,
    R: EndpointState,
    I: Implementation,
{
    local: L,
    remote: R,
    framer: F,
    trans: TransportProperties,
    _phantom: PhantomData<I>,
}

impl EndpointState for NoEndpoint {}
impl<T> EndpointState for T where T: Endpoint {}

impl<F> Preconnection<F, NoEndpoint, NoEndpoint, DefaultImpl>
where
    F: Framer,
{
    /// Create a new Preconnection which has no endpoints specified.
    pub fn new(props: TransportProperties, framer: F) -> Self {
        Preconnection {
            local: NoEndpoint,
            remote: NoEndpoint,
            trans: props,
            framer,
            _phantom: PhantomData,
        }
    }
}

impl<F, I> Preconnection<F, NoEndpoint, NoEndpoint, I>
where
    F: Framer,
    I: Implementation,
{
    pub fn with_impl(props: TransportProperties, framer: F) -> Self {
        Preconnection {
            local: NoEndpoint,
            remote: NoEndpoint,
            trans: props,
            framer,
            _phantom: PhantomData,
        }
    }
}

impl<F, L, R, I> Preconnection<F, L, R, I>
where
    L: EndpointState,
    R: EndpointState,
    F: Framer,
    I: Implementation,
{
    /// Specify the local endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    pub fn local_endpoint<N>(self, local: N) -> Preconnection<F, N, R, I>
    where
        N: Endpoint,
    {
        Preconnection {
            local,
            remote: self.remote,
            trans: self.trans,
            framer: self.framer,
            _phantom: self._phantom,
        }
    }

    /// Specify the remote endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    ///
    /// No name resolution is done by this method.
    pub fn remote_endpoint<N>(self, remote: N) -> Preconnection<F, L, N, I>
    where
        N: Endpoint,
    {
        Preconnection {
            local: self.local,
            remote,
            trans: self.trans,
            framer: self.framer,
            _phantom: self._phantom,
        }
    }

    pub fn transport_properties(&self) -> &TransportProperties {
        &self.trans
    }

    pub fn transport_properties_mut(&mut self) -> &mut TransportProperties {
        &mut self.trans
    }
}

impl<F, R, I> Preconnection<F, NoEndpoint, R, I>
where
    R: Endpoint,
    F: Framer,
    I: Implementation,
{
    pub async fn initiate(self) -> Result<Box<dyn Connection<F>>, Error> {
        I::connection(self.framer, Option::<&'static str>::None, self.remote, self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }
}

impl<F, L, I> Preconnection<F, L, NoEndpoint, I>
where
    L: Endpoint,
    F: Framer + Clone + Unpin,
    I: Implementation,
{
    pub async fn listen(self) -> Result<Box<dyn Listener<F>>, Error> {
        I::listener(self.framer, self.local, Option::<&'static str>::None, self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Listen)
    }
}

impl<F, L, R, I> Preconnection<F, L, R, I>
where
    L: Endpoint,
    R: Endpoint,
    F: Framer + Clone + Unpin,
    I: Implementation,
{
    pub async fn listen(self) -> Result<Box<dyn Listener<F>>, Error> {
        I::listener(self.framer, self.local, Some(self.remote), self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Listen)
    }
}

impl<F, L, R, I> Preconnection<F, L, R, I>
where
    L: Endpoint,
    R: Endpoint,
    F: Framer,
    I: Implementation,
{
    pub async fn initiate(self) -> Result<Box<dyn Connection<F>>, Error> {
        I::connection(self.framer, Some(self.local), self.remote, self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }
}
