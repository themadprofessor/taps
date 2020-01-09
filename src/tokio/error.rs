use snafu::Snafu;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("failed to resolve endpoint: {}", source))]
    Resolve { source: Box<dyn StdError + StdSend> },

    #[snafu(display("endpoint resolved to nothing"))]
    NoEndpoint,

    #[snafu(display("failed to connect to endpoint: {}", source))]
    Connect { source: crate::error::Error },

    #[snafu(display("failed to open connection: {}", source))]
    Open { source: tokio::io::Error },

    #[snafu(display("failed to send message: {}", source))]
    Send { source: tokio::io::Error },

    #[snafu(display("failed to receive message: {}", source))]
    Receive { source: tokio::io::Error },

    #[snafu(display("failed to close connection: {}", source))]
    Close { source: tokio::io::Error },

    #[snafu(display("failed to frame message: {}", source))]
    Frame { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to deframe message: {}", source))]
    Deframe { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to listen for connections: {}", source))]
    Listen { source: tokio::io::Error },
}
