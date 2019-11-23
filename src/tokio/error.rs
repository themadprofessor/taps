use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("failed to resolve endpoint: {}", source))]
    Resolve { source: ::tokio::io::Error },
}
