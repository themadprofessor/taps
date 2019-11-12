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

use serde::export::PhantomData;
use tokio_util::codec::{BytesCodec, Framed};

use async_trait::async_trait;

use crate::Endpoint;
use crate::error::Error;
use crate::properties::TransportProperties;
use crate::tokio::connection::Connection;
use crate::tokio::race::race;

/// A configuration type used to configure how to create a Connection.
///
/// # Generic Types
/// - T: The type which is going to be sent/received over the Connection.
/// - L: The type of the local endpoint.
/// - R: The type of the remote endpoint.
pub struct Preconnection<T, L, R> {
    local: Option<L>,
    remote: Option<R>,
    trans_props: TransportProperties,
    _phantom: PhantomData<T>,
}

impl<T, L, R> Preconnection<T, L, R> {
    /// Create a new Preconnection.
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
            _phantom: self._phantom,
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
impl<T, L, R> crate::Preconnection<T, L, R> for Preconnection<T, L, R>
where
    L: Endpoint + Send,
    R: Endpoint + Send,
{
    fn local_endpoint(&mut self, local: L) {
        self.local = Some(local);
    }

    fn remote_endpoint(&mut self, remote: R) {
        self.remote = Some(remote)
    }

    fn transport_properties(&self) -> &TransportProperties {
        &self.trans_props
    }

    fn transport_properties_mut(&mut self) -> &mut TransportProperties {
        &mut self.trans_props
    }

    async fn initiate<C>(self) -> Result<C, Error>
    where
        C: crate::Connection<T>,
        T: Send + 'static,
    {
        race(self.remote.expect("no remote endpoint given"), &self.trans_props).await
    }
}

/*
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
