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

use crate::error::Error;
use crate::Endpoint;
use crate::properties::TransportProperties;
use crate::tokio::connection::Connection;
use crate::tokio::listener::Listener;
use crate::tokio::race::race;
use serde::export::PhantomData;
use std::fmt;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_dns::ToEndpoint;
use tokio_util::codec::{BytesCodec, Framed};
use async_trait::async_trait;

/// A configuration type used to configure how to create a Connection.
///
/// # Generic Types
/// - T: The type which is going to be sent/received over the Connection
/// - L: The type of the local endpoint. It is [NoEndpoint](struct.NoEndpoint.html) if no local
/// endpoint is given
/// - R: The type of the remote endpoint. It is [NoEndpoint](struct.NoEndpoint.html) if no remote
/// endpoint is given
pub struct Preconnection<T, L, R> {
    local: Option<L>,
    remote: Option<R>,
    trans_props: TransportProperties,
    _phantom: PhantomData<T>,
}

impl<T, L, R> Preconnection<T, L, R> {
    /// Create a new Preconnection which has no endpoints specified.
    pub fn new(props: TransportProperties) -> Self {
        // Note: Box::new does not alloc when given 0-sized type.
        Preconnection {
            local: None,
            remote: None,
            trans_props: props,
            _phantom: PhantomData,
        }
    }
}

impl<T, L, R> fmt::Debug for Preconnection<T, L, R>
where
    L: fmt::Debug,
    R: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Preconnection")
            .field("local", &self.local)
            .field("remote", &self.remote)
            .finish()
    }
}

impl<T, L, R> Clone for Preconnection<T, L, R>
where
    L: Clone,
    R: Clone,
{
    fn clone(&self) -> Self {
        Preconnection {
            local: self.local.clone(),
            remote: self.remote.clone(),
            trans_props: self.trans_props.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

/// Two [Preconnection](struct.Preconnection.html)s are equal if both their local and remote
/// endpoints equal and have the equivalent [TransportProperties](struct.TransportProperties.html).
impl<T, L, R> PartialEq for Preconnection<T, L, R>
where
    L: PartialEq,
    R: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.local == other.local
            && self.remote == other.remote
            && self.trans_props == other.trans_props
    }
}

#[async_trait]
impl <T, L, R> crate::Preconnection<T> for Preconnection<T, L, R> {
    fn local_endpoint<N>(&mut self, local: N) where N: Endpoint {
        self.local = Some(local);
    }

    fn remote_endpoint<N>(&mut self, remote: N) where N: Endpoint {
        self.remote = Some(remote)
    }

    async fn initiate<C>(self) -> C where C: crate::Connection<T>, T: Send + 'static {
        unimplemented!()
    }
}

/*impl<T, L, R> Preconnection<T, L, R>
    where
        L: EndpointState,
        R: EndpointState,
{
    /// Specify the local endpoint which will be used when creating a Connection from this
    /// [Preconnection](struct.Preconnection.html).
    pub fn local_endpoint<'a, N>(self, local: N) -> Preconnection<T, N, R>
        where
            N: ToEndpoint<'a>,
    {
        Preconnection {
            local,
            remote: self.remote,
            trans_props: self.trans_props,
            _phantom: self._phantom,
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
    pub fn remote_endpoint<'a, N>(self, remote: N) -> Preconnection<T, L, N>
        where
            N: ToEndpoint<'a>,
    {
        Preconnection {
            local: self.local,
            remote,
            trans_props: self.trans_props,
            _phantom: self._phantom,
        }
    }

    pub fn transport_properties(&self) -> &TransportProperties {
        &self.trans_props
    }

    pub fn transport_properties_mut(&mut self) -> &mut TransportProperties {
        &mut self.trans_props
    }
}

impl<'a, T, L, R> Preconnection<T, L, R>
    where
        L: EndpointState,
        R: ToEndpoint<'a>,
{
    pub async fn initiate(self) -> Result<Connection<TcpStream, BytesCodec>, Error> {
        let conn = race(self.remote, &self.trans_props).await?;
        Ok(Connection {
            conn: Framed::new(conn, BytesCodec::new()),
        })
    }
}

impl<'a, T, L, R> Preconnection<T, L, R>
    where
        L: ToEndpoint<'a>,
        R: EndpointState,
{
    pub async fn listen(self) -> Result<Listener<::tokio::net::tcp::Incoming>, Error> {
        Listener::<::tokio::net::tcp::Incoming>::create(self.local, &self.trans_props).await
    }
}*/
