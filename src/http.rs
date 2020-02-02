//! A naive HTTP framer implementation built upon the `http` crate.

use crate::error::box_error;
use crate::frame::Framer;
use crate::{Decode, Encode};
use bytes::{Buf, BytesMut};
use http::header::HeaderName;
use http::version::Version as HttpVersion;
use http::{HeaderMap, HeaderValue, Request, Response};
use log::{debug, trace};
use snafu::{OptionExt, ResultExt, Snafu};
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::marker::Send as StdSend;

/// Naive HTTP framer implementation. **NOT PRODUCTION SAFE**
#[derive(Debug, Clone, Default)]
pub struct Http<T> {
    headers: HeaderMap,
    _data: PhantomData<T>,
}

/// Errors which the [Http](struct.Http.html) framer can return.
#[derive(Debug, Snafu)]
pub enum HttpError {
    /// No empty line is found to signify the end of the HTTP headers.
    #[snafu(display("no empty line found"))]
    MissingEmptyLine,

    /// No HTTP status line found.
    #[snafu(display("no status line found"))]
    MissingStatusLine,

    /// The HTTP status line is malformed. E.G. invalid HTTP version.
    #[snafu(display("malformed status line"))]
    MalformedStatusLine,

    /// A received header was deemed invalid by the `http` crate.
    #[snafu(display("invalid header: {}", source))]
    InvalidHeader { source: http::Error },

    /// The body could not be encoded if sending or decoded if receiving.
    #[snafu(display("invalid body: {}", source))]
    InvalidBody { source: Box<dyn StdError + StdSend> },
}

impl<T> Encode for Request<T>
where
    T: Encode,
    <T as Encode>::Error: 'static,
{
    type Error = HttpError;

    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
        let req = self;
        trace!("request method: {}", req.method());
        trace!("request uri: {}", req.uri());
        trace!("request version: {:?}", req.version());
        data.extend_from_slice(req.method().as_str().as_bytes());
        data.extend_from_slice(&[b' ']);
        data.extend_from_slice(
            req.uri()
                .path_and_query()
                .map(|p| p.as_str())
                .unwrap_or_else(|| "/")
                .as_bytes(),
        );
        data.extend_from_slice(&[b' ']);
        data.extend_from_slice(version_to_string(req.version()).as_bytes());
        data.extend_from_slice(b"\r\n");

        debug!("headers count: {}", req.headers().len());
        trace!("http headers: {:?}", req.headers());
        for (header, value) in req.headers().iter() {
            data.extend_from_slice(header.as_ref());
            data.extend_from_slice(&[b':', b' ']);
            data.extend_from_slice(value.as_bytes());
            data.extend_from_slice(b"\r\n");
        }
        data.extend_from_slice(b"\r\n");
        trace!("headers bytes: {}", data.len());

        req.body()
            .encode(data)
            .map_err(box_error)
            .with_context(|| InvalidBody)
    }
}

impl<T> Decode for Response<T>
where
    T: Decode,
    <T as Decode>::Error: 'static,
{
    type Error = HttpError;

    fn decode(data: &mut BytesMut) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut response = Response::builder();
        let full_response = String::from_utf8_lossy(data);
        let mut iter = full_response.split("\r\n");

        let mut status_line = iter
            .next()
            .and_then(|l| if l.is_empty() { None } else { Some(l) })
            .with_context(|| MissingStatusLine)?
            .split(' ');

        response = response.version(status_line_to_version(
            status_line.next().with_context(|| MalformedStatusLine)?,
        )?);
        response = response.status(status_line.next().with_context(|| MalformedStatusLine)?);

        for line in iter {
            // Stop at first empty line as this signals end of headers
            if line.is_empty() {
                break;
            }

            let mut split = line.split(':');
            let name = split.next();
            let value = split.next();

            if let Some(n) = name {
                if let Some(v) = value {
                    response = response.header(n, v);
                }
            }
        }

        let header_len = full_response
            .find("\r\n\r\n")
            .with_context(|| MissingEmptyLine)?
            + 4; // Find returns the index of the first char in the pattern
        data.advance(header_len);
        let body = T::decode(data)
            .map_err(box_error)
            .with_context(|| InvalidBody)?;

        response.body(body).with_context(|| InvalidHeader)
    }
}

impl<T> Framer for Http<T>
where
    T: Encode + Decode + Send + 'static,
    <T as Encode>::Error: 'static,
    <T as Decode>::Error: 'static,
{
    type Input = Request<T>;
    type Output = Response<T>;
    type MetaKey = HeaderName;
    type MetaValue = HeaderValue;
    type Error = HttpError;

    fn frame(&mut self, item: Self::Input, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst)
    }

    fn deframe(&mut self, src: &mut BytesMut) -> Result<Option<Self::Output>, Self::Error> {
        Self::Output::decode(src).map(Some)
    }

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue) {
        self.headers.insert(key, value);
    }
}

fn status_line_to_version(status_line: &str) -> Result<HttpVersion, HttpError> {
    let ver_str = status_line
        .split('/')
        .nth(1)
        .with_context(|| MalformedStatusLine)?;

    match ver_str {
        "0.9" => Ok(HttpVersion::HTTP_09),
        "1.0" => Ok(HttpVersion::HTTP_10),
        "1.1" => Ok(HttpVersion::HTTP_11),
        "2.0" => Ok(HttpVersion::HTTP_2),
        "3.0" => Ok(HttpVersion::HTTP_3),
        _ => Err(HttpError::MalformedStatusLine),
    }
}

fn version_to_string(version: HttpVersion) -> &'static str {
    match version {
        HttpVersion::HTTP_09 => "HTTP/0.9",
        HttpVersion::HTTP_10 => "HTTP/1.0",
        HttpVersion::HTTP_11 => "HTTP/1.1",
        HttpVersion::HTTP_2 => "HTTP/2.0",
        HttpVersion::HTTP_3 => "HTTP/3.0",
        _ => unreachable!(),
    }
}
