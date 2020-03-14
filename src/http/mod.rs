use bytes::{Buf, BytesMut};
use http::header::{HeaderName, InvalidHeaderName, InvalidHeaderValue};
use http::status::InvalidStatusCode;
use http::{HeaderValue, Request, StatusCode, Version, Response};
use log::trace;
use snafu::Snafu;

mod client;
mod server;
pub use client::HttpClient;
pub use server::HttpServer;

#[derive(Debug, Snafu)]
pub enum InvalidHeaderError {
    #[snafu(display("missing colon"))]
    MissingColon,

    #[snafu(display("{}", source))]
    InvalidName { source: InvalidHeaderName },

    #[snafu(display("{}", source))]
    InvalidValue { source: InvalidHeaderValue },
}

#[derive(Debug, Snafu)]
pub enum InvalidContentLengthError {
    #[snafu(display("{}", source))]
    InvalidNumber { source: std::num::ParseIntError },

    #[snafu(display("{}", source))]
    InvalidUtf8 { source: std::str::Utf8Error },
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("no host header or uri specified"))]
    NoHost,

    #[snafu(display("invalid host: {}", source))]
    InvalidHost {
        source: http::header::InvalidHeaderValue,
    },

    #[snafu(display("failed to encode body, {}", source))]
    Body {
        source: Box<dyn ::std::error::Error + Send>,
    },

    #[snafu(display("invalid version"))]
    InvalidVersion,

    #[snafu(display("invalid status code: {}", source))]
    InvalidStatus { source: InvalidStatusCode },

    #[snafu(display("invalid header: {}", source))]
    InvalidHeader { source: InvalidHeaderError },

    #[snafu(display("invalid response: {}", source))]
    InvalidResponse { source: http::Error },

    #[snafu(display("invalid request: {}", source))]
    InvalidRequest { source: http::Error },

    #[snafu(display("invalid request line"))]
    InvalidRequestLine,

    #[snafu(display("invalid content length: {}", source))]
    InvalidContentLength { source: InvalidContentLengthError },

    #[snafu(display("invalid body: {}", source))]
    InvalidBody {
        source: Box<dyn ::std::error::Error + Send>,
    },
}



fn bytes_to_ver(raw: &[u8]) -> Result<Version, Error> {
    match raw {
        b"0.9" => Ok(Version::HTTP_09),
        b"1.0" => Ok(Version::HTTP_10),
        b"1.1" => Ok(Version::HTTP_11),
        b"2.0" => Ok(Version::HTTP_2),
        b"3.0" => Ok(Version::HTTP_3),
        _ => Err(Error::InvalidVersion),
    }
}

fn version_bytes(ver: Version) -> &'static [u8] {
    match ver {
        Version::HTTP_09 => b"HTTP/0.9",
        Version::HTTP_10 => b"HTTP/1.0",
        Version::HTTP_11 => b"HTTP/1.1",
        Version::HTTP_2 => b"HTTP/2.0",
        Version::HTTP_3 => b"HTTP/3.0",
        _ => panic!("invalid http version"),
    }
}

fn write_request_line<T>(req: &Request<T>, data: &mut BytesMut) {
    data.extend_from_slice(req.method().as_str().as_bytes());
    data.extend_from_slice(b" ");
    data.extend_from_slice(
        req.uri()
            .path_and_query()
            .map(|s| s.as_str())
            .unwrap_or_else(|| "/")
            .as_bytes(),
    );
    data.extend_from_slice(&[b' ']);
    data.extend_from_slice(version_bytes(req.version()));
    data.extend_from_slice(b"\r\n");
}

fn write_response_line<T>(res: &Response<T>, data: &mut BytesMut) {
    data.extend_from_slice(version_bytes(res.version()));
    data.extend_from_slice(b" ");
    data.extend_from_slice(res.status().as_str().as_bytes());
    data.extend_from_slice(res.status().canonical_reason().unwrap_or_default().as_bytes());
    data.extend_from_slice(b"\r\n");
}

fn write_header(header: (&HeaderName, &HeaderValue), data: &mut BytesMut) {
    data.extend_from_slice(header.0.as_ref());
    data.extend_from_slice(b":");
    data.extend_from_slice(header.1.as_bytes());
    data.extend_from_slice(b"\r\n");
}
