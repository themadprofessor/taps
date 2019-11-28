use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("endpoint resolved into nothing"))]
    NoEndpoint,
}
