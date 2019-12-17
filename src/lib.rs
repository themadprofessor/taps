#![allow(dead_code)]
#![forbid(unsafe_code)]

pub use connection::Connection;
pub use frame::Framer;
pub use preconnection::*;
pub use encode::*;

use crate::error::Error;
use crate::properties::TransportProperties;

mod connection;
pub mod error;
mod encode;
mod frame;
pub mod http;
mod preconnection;
pub mod properties;
mod tokio;

/// Create a new [Preconnection](trait.Preconnection.html).
///
/// This is the main entry point of TAPS.
///
/// # Example
///
/// ```
/// use taps::properties::TransportProperties;
/// use taps::prelude::*;
/// use std::net::{SocketAddr, SocketAddrV4};
/// use taps::http::Http;
/// use std::str::FromStr;
/// let mut preconnection = taps::new_preconnection::<(), _, Http<String>>(TransportProperties::default());
/// preconnection.remote_endpoint(SocketAddr::from_str("1.1.1.1:80").unwrap());
/// ```
pub fn new_preconnection<L, R, F>(props: TransportProperties) -> impl Preconnection<L, R, F>
where
    L: Endpoint + Send,
    R: Endpoint + Send,
    F: Framer + Send + Sync + Clone + 'static,
{
    crate::tokio::preconnection::Preconnection::new(props)
}

pub mod prelude {
    pub use crate::preconnection::{Endpoint, Preconnection};
    pub use crate::connection::Connection;
    pub use crate::frame::Framer;
    pub use crate::encode::*;
}
