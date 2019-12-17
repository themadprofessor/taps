use crate::connection::Connection;
use crate::frame::Framer;
use crate::properties::TransportProperties;
use async_trait::async_trait;
use std::error::Error as StdError;
use std::marker::Send as StdSend;
use std::net::SocketAddr;
use std::fmt;

#[derive(Debug)]
pub struct MissingEndpoint;

impl fmt::Display for MissingEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("missing endpoint")
    }
}

impl StdError for MissingEndpoint {}

/// The `Endpoint` trait allows resolving a domain name into `SocketAddr`s.
#[async_trait]
pub trait Endpoint {
    type Error: StdSend + StdError;

    async fn resolve(self) -> Result<Vec<SocketAddr>, Self::Error>;
}

#[async_trait]
impl Endpoint for SocketAddr {
    type Error = ::std::convert::Infallible;

    async fn resolve(self) -> Result<Vec<SocketAddr>, Self::Error> {
        Ok(vec![self])
    }
}

#[async_trait]
impl Endpoint for () {
    type Error = MissingEndpoint;

    async fn resolve(self) -> Result<Vec<SocketAddr>, Self::Error> {
        Err(MissingEndpoint)
    }
}

#[async_trait]
pub trait Preconnection<L, R, F> {
    type Error: StdSend + StdError;

    fn local_endpoint(&mut self, local: L)
    where
        L: Endpoint;

    fn remote_endpoint(&mut self, remote: R)
    where
        R: Endpoint;

    fn transport_properties(&self) -> &TransportProperties;

    fn transport_properties_mut(&mut self) -> &mut TransportProperties;

    fn add_framer(&mut self, framer: F);

    async fn initiate(self) -> Result<Box<dyn Connection<F, Error = Self::Error>>, Self::Error>
    where
        R: Endpoint + Send,
        <R as Endpoint>::Error: 'static,
        F: Framer + Send,
        F::Input: ::std::marker::Send;
}
