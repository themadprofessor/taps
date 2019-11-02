use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("failed to resolve endpoint: {}", source))]
    Resolution { source: ::std::io::Error },
    #[snafu(display("failed to connect to endpoint: {}", source))]
    Connecting { source: ::std::io::Error },
}
