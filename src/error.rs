use snafu::Snafu;
use std::error::Error as StdError;
use std::marker::Send as StdSend;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("failed to initiate connection: {}", source))]
    Initiate { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to begin listening for connections: {}", source))]
    Listen { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to rendezvous: {}", source))]
    Rendezvous { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to send data: {}", source))]
    Send { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to receive data: {}", source))]
    Receive { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to encode data: {}", source))]
    Encode { source: Box<dyn StdError + StdSend> },

    #[snafu(display("failed to decode data: {}", source))]
    Decode { source: Box<dyn StdError + StdSend> },

    #[snafu(display("connection terminated unexpectedly: {}", source))]
    Connection { source: Box<dyn StdError + StdSend> },
}

/// A utility function which boxes the given error, and returns a trait object which can be used
/// with [Error](enum.Error.html).
pub fn box_error<T>(error: T) -> Box<dyn StdError + StdSend>
where
    T: StdError + StdSend + 'static,
{
    Box::new(error) as Box<dyn StdError + StdSend>
}
