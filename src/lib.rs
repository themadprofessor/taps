#![allow(dead_code)]
#![forbid(unsafe_code)]

pub use connection::Connection;
pub use encode::*;
pub use frame::Framer;
pub use listener::Listener;
pub use preconnection::Preconnection;
pub use resolve::Endpoint;

mod connection;
mod encode;
pub mod error;
mod frame;
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
    pub use crate::encode::*;
    pub use crate::frame::Framer;
    pub use crate::listener::Listener;
    pub use crate::preconnection::Preconnection;
    pub use crate::resolve::Endpoint;
}
