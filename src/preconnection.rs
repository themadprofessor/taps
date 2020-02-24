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
pub struct Preconnection<F, L, R, SD, RD, I = DefaultImpl>
where
    F: Framer<SD, RD>,
    L: EndpointState,
    R: EndpointState,
    I: Implementation,
{
    local: L,
    remote: R,
    framer: F,
    trans: TransportProperties,
    _phantom: PhantomData<I>,
    _send: PhantomData<SD>,
    _recv: PhantomData<RD>,
}

impl EndpointState for NoEndpoint {}
impl<T> EndpointState for T where T: Endpoint {}

impl<F, SD, RD> Preconnection<F, NoEndpoint, NoEndpoint, SD, RD, DefaultImpl>
where
    F: Framer<SD, RD>,
{
    /// Create a new Preconnection which has no endpoints specified.
    pub fn new(props: TransportProperties, framer: F) -> Self {
        Preconnection {
            local: NoEndpoint,
            remote: NoEndpoint,
            trans: props,
            framer,
            _phantom: PhantomData,
            _send: PhantomData,
            _recv: PhantomData,
        }
    }
}

impl<F, SD, RD, I> Preconnection<F, NoEndpoint, NoEndpoint, SD, RD, I>
where
    F: Framer<SD, RD>,
    I: Implementation,
{
    pub fn with_impl(props: TransportProperties, framer: F) -> Self {
        Preconnection {
            local: NoEndpoint,
            remote: NoEndpoint,
            trans: props,
            framer,
            _phantom: PhantomData,
            _send: PhantomData,
            _recv: PhantomData,
        }
    }
}

impl<F, L, R, SD, RD, I> Preconnection<F, L, R, SD, RD, I>
where
    L: EndpointState,
    R: EndpointState,
    F: Framer<SD, RD>,
    I: Implementation,
{
    /// Specify the local endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    pub fn local_endpoint<N>(self, local: N) -> Preconnection<F, N, R, SD, RD, I>
    where
        N: Endpoint,
    {
        Preconnection {
            local,
            remote: self.remote,
            trans: self.trans,
            framer: self.framer,
            _phantom: self._phantom,
            _send: PhantomData,
            _recv: PhantomData,
        }
    }

    /// Specify the remote endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    ///
    /// No name resolution is done by this method.
    pub fn remote_endpoint<N>(self, remote: N) -> Preconnection<F, L, N, SD, RD, I>
    where
        N: Endpoint,
    {
        Preconnection {
            local: self.local,
            remote,
            trans: self.trans,
            framer: self.framer,
            _phantom: self._phantom,
            _send: PhantomData,
            _recv: PhantomData,
        }
    }

    pub fn transport_properties(&self) -> &TransportProperties {
        &self.trans
    }

    pub fn transport_properties_mut(&mut self) -> &mut TransportProperties {
        &mut self.trans
    }
}

impl<F, R, SD, RD, I> Preconnection<F, NoEndpoint, R, SD, RD, I>
where
    R: Endpoint,
    F: Framer<SD, RD> + Clone,
    I: Implementation,
    SD: Send + 'static,
    RD: Send + 'static,
{
    pub async fn initiate(self) -> Result<Box<dyn Connection<F, SD, RD>>, Error> {
        I::connection(self.framer, Option::<()>::None, self.remote, &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }
}

impl<F, L, SD, RD, I> Preconnection<F, L, NoEndpoint, SD, RD, I>
where
    L: Endpoint,
    F: Framer<SD, RD>,
    I: Implementation,
{
    pub async fn listen(self) -> Result<Box<dyn Listener<F, SD, RD>>, Error> {
        I::listener(self.framer, self.local, Option::<()>::None, &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Listen)
    }
}

impl<F, L, R, SD, RD, I> Preconnection<F, L, R, SD, RD, I>
where
    L: Endpoint,
    R: Endpoint,
    F: Framer<SD, RD>,
    I: Implementation,
{
    pub async fn listen(self) -> Result<Box<dyn Listener<F, SD, RD>>, Error> {
        I::listener(self.framer, self.local, Some(self.remote), &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Listen)
    }
}

impl<F, L, R, SD, RD, I> Preconnection<F, L, R, SD, RD, I>
where
    L: Endpoint,
    R: Endpoint,
    F: Framer<SD, RD> + Clone,
    I: Implementation,
    SD: Send + 'static,
    RD: Send + 'static,
{
    pub async fn initiate(self) -> Result<Box<dyn Connection<F, SD, RD>>, Error> {
        I::connection(self.framer, Some(self.local), self.remote, &self.trans)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Initiate)
    }
}
