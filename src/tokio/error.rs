use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("failed to resolve endpoint: {}", source))]
    Resolution { source: crate::error::Error },
    #[snafu(display("no endpoints resolved"))]
    NoEndpoint,
    #[snafu(display("failed to connect to endpoint: {}", source))]
    Connecting { source: ::std::io::Error },
    #[snafu(display("failed to bind to endpoint: {}", source))]
    Binding { source: ::tokio::io::Error },
}
