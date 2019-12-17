#![allow(dead_code)]
#![forbid(unsafe_code)]

pub use connection::Connection;
pub use encode::*;
pub use frame::Framer;
pub use preconnection::{Endpoint, Preconnection};

use crate::properties::TransportProperties;

mod connection;
mod encode;
pub mod error;
mod frame;
pub mod http;
mod preconnection;
pub mod properties;
pub mod tokio;

/// Create a new [Preconnection](trait.Preconnection.html).
///
/// This is the main entry point of TAPS.
///
/// # Example
///
/// ```
/// # use std::net::{SocketAddr, SocketAddrV4};
/// # use std::str::FromStr;
/// use taps::properties::TransportProperties;
/// use taps::prelude::*;
/// use taps::http::Http;
/// let mut preconnection = taps::new_preconnection::<(), _, Http<String>>(TransportProperties::default());
/// preconnection.remote_endpoint(SocketAddr::from_str("1.1.1.1:80").unwrap());
/// ```
pub fn new_preconnection<L, R, F>(props: TransportProperties) -> impl Preconnection<L, R, F>
where
    L: Endpoint + Send,
    R: Endpoint + Send,
    F: Framer + Send + Sync + Clone + 'static,
{
    crate::tokio::Preconnection::new(props)
}

/// TAPS prelude, intended for glob imports.
pub mod prelude {
    pub use crate::connection::Connection;
    pub use crate::encode::*;
    pub use crate::frame::Framer;
    pub use crate::preconnection::{Endpoint, Preconnection};
}
