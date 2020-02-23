#![allow(dead_code)]
#![forbid(unsafe_code)]

pub use connection::Connection;
pub use codec::*;
pub use listener::Listener;
pub use preconnection::Preconnection;
pub use resolve::Endpoint;

mod connection;
pub mod codec;
pub mod error;
pub mod http;
mod implementation;
mod listener;
mod preconnection;
pub mod properties;
mod resolve;

#[cfg(feature = "tokio-impl")]
pub mod tokio;

/// TAPS prelude, intended for glob imports.
pub mod prelude {
    pub use crate::connection::Connection;
    pub use crate::codec::*;
    pub use crate::listener::Listener;
    pub use crate::preconnection::Preconnection;
    pub use crate::resolve::Endpoint;
}
