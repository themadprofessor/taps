use crate::codec::Error as CodecError;
use crate::error::box_error;
use crate::{Decode, Encode};
use bytes::{Buf, BytesMut};
use http::header::{HeaderName, InvalidHeaderName, InvalidHeaderValue};
use http::response::Builder;
use http::status::InvalidStatusCode;
use http::{HeaderValue, Request, Response, StatusCode, Version};
use snafu::{ResultExt, Snafu};
use std::convert::TryInto;

pub struct Http;

#[derive(Debug, Default)]
struct DecodeState<T> {
    builder: Builder,
    state: State,
    body_state: T,
}

#[derive(Debug)]
enum State {
    Status,
    Headers,
    Body,
}

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

    #[snafu(display("invalid body: {}", source))]
    InvalidBody {
        source: Box<dyn ::std::error::Error + Send>,
    },
}

impl<T> Decode for Response<T>
where
    T: Decode,
    <T as Decode>::Error: 'static,
{
    type Error = Error;
    type State = DecodeState<T::State>;

    fn decode(data: &mut BytesMut, state: Self::State) -> Result<Self, CodecError<Self::Error>>
    where
        Self: Sized,
    {
        loop {
            match state.state {
                State::Status => read_status(data, state)?,
                State::Headers => read_header(data, state)?,
                State::Body => {
                    return match T::decode(data, state.body_state) {
                        Ok(x) => state
                            .builder
                            .body(x)
                            .with_context(|| InvalidResponse)
                            .map_err(|e| CodecError::Err(e)),
                        Err(e) => Err(box_error(e))
                            .with_context(|| InvalidBody)
                            .map_err(|e| CodecError::Err(e)),
                    }
                }
            }
        }
    }
}

impl<T> Encode for Request<T>
where
    T: Encode,
{
    type Error = Error;

    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
        write_request_line(self, data);
        if !self.headers().contains_key(http::header::HOST) {
            let host = self
                .uri()
                .host()
                .ok_or_else(|| Error::NoHost)?
                .try_into()
                .with_context(|| InvalidHost)?;
            let header = http::header::HOST;
            write_header((&header, &host), data)
        }

        for x in self.headers().iter() {
            write_header(x, data);
        }

        data.extend_from_slice(b"\r\n\r\n");

        self.body()
            .encode(data)
            .map_err(crate::error::box_error)
            .with_context(|| Body)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl Default for State {
    fn default() -> Self {
        State::Status
    }
}

impl From<Error> for CodecError<Error> {
    fn from(e: Error) -> Self {
        CodecError::Err(e)
    }
}

fn read_status<T>(
    data: &mut BytesMut,
    state: &mut DecodeState<T>,
) -> Result<(), CodecError<Error>> {
    let mut raw_status = find_eol(data).ok_or_else(|| CodecError::Incomplete)?;

    let http_start = raw_status
        .windows(5)
        .enumerate()
        .find(|x| x.1 == b"HTTP/")
        .ok_or_else(|| CodecError::Incomplete)?
        .0;
    raw_status.advance(http_start + 5);
    state.builder = state.builder.version(bytes_to_ver(&raw_status[0..3])?);
    raw_status.advance(4); // Skip number and space

    state.builder = state
        .builder
        .status(StatusCode::from_bytes(&raw_status[0..3]).with_context(|| InvalidStatus)?);

    state.state = State::Headers;

    Ok(())
}

fn read_header<T>(
    data: &mut BytesMut,
    state: &mut DecodeState<T>,
) -> Result<(), CodecError<Error>> {
    let raw_header = find_eol(data).ok_or_else(|| CodecError::Incomplete)?;

    // Reached empty line
    if raw_header.len() == 0 && data.starts_with(b"\r\n") {
        data.advance(2);
        state.state = State::Body;
        return Ok(());
    }

    let split = raw_header
        .iter()
        .enumerate()
        .find(|x| x.1 == &b':')
        .map(|x| x.0)
        .ok_or_else(|| Error::InvalidHeader {
            source: InvalidHeaderError::MissingColon,
        })?;
    let (name, value) = raw_header.split_at(split);

    // Skip the colon in value
    state.builder = state.builder.header(name, &value[1..]);
    Ok(())
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

fn find_eol(data: &mut BytesMut) -> Option<BytesMut> {
    let i = data
        .windows(2)
        .enumerate()
        .find(|x| x.1 == &b"\r\n"[..])
        .map(|x| x.0)?;

    let mut raw = data.split_to(i);
    data.advance(2); // Skip the newline and return

    Some(raw)
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

fn write_header(header: (&HeaderName, &HeaderValue), data: &mut BytesMut) {
    data.extend_from_slice(header.0.as_ref());
    data.extend_from_slice(b":");
    data.extend_from_slice(header.1.as_bytes());
    data.extend_from_slice(b"\r\n");
}
