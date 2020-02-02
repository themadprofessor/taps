use crate::connection::Connection;
use crate::error::{Error, box_error};
use crate::frame::Framer;
use crate::properties::TransportProperties;
use crate::resolve::Endpoint;
use crate::tokio::Tokio;
use std::marker::PhantomData;
use crate::implementation::Impl;
use snafu::ResultExt;
use crate::Listener;

pub type DefaultImpl = Tokio;

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
pub struct Preconnection<F, L, R, I=DefaultImpl>
where
    F: Framer + Send + 'static,
    L: EndpointState,
    R: EndpointState,
    I: Impl
{
    local: L,
    remote: R,
    framer: F,
    trans: TransportProperties,
    _phantom: PhantomData<I>
}

impl EndpointState for NoEndpoint {}
impl <T> EndpointState for T where T: Endpoint {}

impl<F, I> Preconnection<F, NoEndpoint, NoEndpoint, I>
where
    F: Framer + Send + 'static,
    I: Impl
{
    /// Create a new Preconnection which has no endpoints specified.
    pub fn new(props: TransportProperties, framer: F) -> Self {
        Preconnection {
            local: NoEndpoint,
            remote: NoEndpoint,
            trans: props,
            framer,
            _phantom: PhantomData
        }
    }
}

impl<F, L, R, I> Preconnection<F, L, R, I>
where
    L: EndpointState,
    R: EndpointState,
    F: Framer + Send + 'static,
    I: Impl
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
            _phantom: self._phantom
        }
    }

    /// Specify the remote endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    ///
    /// No name resolution is done by this method. See resolve for eager resolution or initiate for
    /// late resolution.
    pub fn remote_endpoint<N>(self, remote: N) -> Preconnection<F, L, N, I>
    where
        N: Endpoint,
    {
        Preconnection {
            local: self.local,
            remote,
            trans: self.trans,
            framer: self.framer,
            _phantom: self._phantom
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
    F: Framer + Clone,
    I: Impl
{
    pub async fn initiate(self) -> Result<Box<dyn Connection<F>>, Error> {
        I::connection(self.framer, Option::<()>::None, self.remote, &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }
}

impl <F, L, I> Preconnection<F, L, NoEndpoint, I>
where
    L: Endpoint,
    F: Framer,
    I: Impl
{
    pub async fn listen(self) -> Result<Box<dyn Listener<F>>, Error> {
        I::listener(self.framer, self.local, Option::<()>::None, &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Listen)
    }
}

impl <F, L, R, I> Preconnection<F, L, R, I>
where
    L: Endpoint,
    R: Endpoint,
    F: Framer,
    I: Impl
{
    pub async fn listen(self) -> Result<Box<dyn Listener<F>>, Error> {
        I::listener(self.framer, self.local, Some(self.remote), &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Listen)
    }
}

impl <F, L, R, I> Preconnection<F, L, R, I>
    where
        L: Endpoint,
        R: Endpoint,
        F: Framer + Clone,
        I: Impl
{
    pub async fn initiate(self) -> Result<Box<dyn Connection<F>>, Error> {
        I::connection(self.framer, Some(self.local), self.remote, &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }
}
