use crate::error::Decode as DecodeError;
use crate::error::{box_error, Error};
use crate::frame::Framer;
use crate::{Decode, Encode};
use bytes::{Buf, Bytes, BytesMut};
use http::header::HeaderName;
use http::version::Version as HttpVersion;
use http::{HeaderMap, HeaderValue, Request, Response};
use snafu::{ResultExt, Snafu};
use std::marker::PhantomData;

#[derive(Debug, Clone, Default)]
pub struct Http<T> {
    headers: HeaderMap,
    _data: PhantomData<T>,
}

#[derive(Debug, Snafu)]
pub enum HttpError {
    #[snafu(display("no empty line found"))]
    MissingEmptyLine,

    #[snafu(display("no status line found"))]
    MissingStatusLine,

    #[snafu(display("malformed status line"))]
    MalformedStatusLine,
}

impl<T> Encode for Request<T>
where
    T: Encode,
{
    fn encode(&self, data: &mut BytesMut) -> Result<(), Error> {
        let req = self;
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

        for (header, value) in req.headers().iter() {
            data.extend_from_slice(header.as_ref());
            data.extend_from_slice(&[b':', b' ']);
            data.extend_from_slice(value.as_bytes());
            data.extend_from_slice(b"\r\n");
        }
        data.extend_from_slice(b"\r\n");

        req.body().encode(data)
    }
}

impl<T> Decode for Response<T>
where
    T: Decode,
{
    fn decode(data: &mut BytesMut) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut response = Response::builder();
        let full_response = String::from_utf8_lossy(data);
        let mut iter = full_response.split("\r\n");

        let mut status_line = iter
            .next()
            .and_then(|l| if l.is_empty() { None } else { Some(l) })
            .ok_or_else(|| box_error(HttpError::MissingStatusLine))
            .with_context(|| DecodeError)?
            .split(' ');

        response = response.version(status_line_to_version(
            status_line
                .next()
                .ok_or_else(|| box_error(HttpError::MalformedStatusLine))
                .with_context(|| DecodeError)?,
        )?);
        response = response.status(
            status_line
                .next()
                .ok_or_else(|| box_error(HttpError::MalformedStatusLine))
                .with_context(|| DecodeError)?,
        );

        for line in iter {
            if line.is_empty() {
                break;
            }

            let mut split = line.split(':');
            let name = split.next();
            let value = split.next();

            if name.is_none() || value.is_none() {
                continue;
            }

            response = response.header(name.unwrap(), value.unwrap());
        }

        let header_len = full_response
            .find("\r\n\r\n")
            .ok_or_else(|| box_error(HttpError::MissingEmptyLine))
            .with_context(|| DecodeError)?
            + 4;
        data.advance(header_len);
        let body = T::decode(data)?;

        response
            .body(body)
            .map_err(box_error)
            .with_context(|| DecodeError)
    }
}

impl<T> Framer for Http<T>
where
    T: Encode + Decode,
{
    type Input = Request<T>;
    type Output = Response<T>;
    type MetaKey = HeaderName;
    type MetaValue = HeaderValue;

    fn frame(&mut self, item: Self::Input, dst: &mut BytesMut) -> Result<(), Error> {
        item.encode(dst)
    }

    fn deframe(&mut self, src: &mut BytesMut) -> Result<Option<Self::Output>, Error> {
        Self::Output::decode(src).map(Some)
    }

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue) {
        self.headers.insert(key, value);
    }
}

fn status_line_to_version(status_line: &str) -> Result<HttpVersion, Error> {
    let ver_str = status_line
        .split('/')
        .nth(1)
        .ok_or_else(|| box_error(HttpError::MalformedStatusLine))
        .with_context(|| DecodeError)?;

    match ver_str {
        "0.9" => Ok(HttpVersion::HTTP_09),
        "1.0" => Ok(HttpVersion::HTTP_10),
        "1.1" => Ok(HttpVersion::HTTP_11),
        "2.0" => Ok(HttpVersion::HTTP_2),
        "3.0" => Ok(HttpVersion::HTTP_3),
        _ => Err(Error::Decode {
            source: box_error(HttpError::MalformedStatusLine),
        }),
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
