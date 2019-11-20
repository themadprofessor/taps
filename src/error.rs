use snafu::Snafu;
use std::error::Error as StdError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("failed to initiate connection: {}", source))]
    Initiate { source: Box<dyn StdError> },

    #[snafu(display("failed to begin listening for connections: {}", source))]
    Listen { source: Box<dyn StdError> },

    #[snafu(display("failed to rendezvous: {}", source))]
    Rendezvous { source: Box<dyn StdError> },

    #[snafu(display("failed to send data: {}", source))]
    Send { source: Box<dyn StdError> },

    #[snafu(display("failed to receive data: {}", source))]
    Receive { source: Box<dyn StdError> },

    #[snafu(display("failed to encode data: {}", source))]
    Encode { source: Box<dyn StdError> },
}
