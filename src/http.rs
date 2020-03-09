use crate::codec::DecodeError;
use crate::error::box_error;
use crate::{Decode, DeframeError, Encode, Framer};
use bytes::{Buf, BytesMut};
use http::header::{HeaderName, InvalidHeaderName, InvalidHeaderValue};
use http::response::Builder;
use http::status::InvalidStatusCode;
use http::{HeaderValue, Request, Response, StatusCode, Version};
use log::{debug, trace};
use snafu::{ResultExt, Snafu};
use std::convert::TryInto;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Http<S, R>
where
    R: Decode + Send + Sync,
{
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
    decode_state: Option<DecodeState<R::State>>,
}

#[derive(Debug, Default)]
pub struct DecodeState<T> {
    builder: Builder,
    state: State,
    content_len: Option<usize>,
    body_state: T,
}

#[derive(Debug, Copy, Clone)]
pub enum State {
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

    #[snafu(display("invalid content length: {}", source))]
    InvalidContentLength { source: InvalidContentLengthError },

    #[snafu(display("invalid body: {}", source))]
    InvalidBody {
        source: Box<dyn ::std::error::Error + Send>,
    },
}

impl<S, R> Framer for Http<S, R>
where
    S: Encode + Send + Sync + 'static,
    R: Decode + Send + Sync + 'static,
    <R as Decode>::State: Send + Sync,
{
    type Input = Request<S>;
    type Output = Response<R>;
    type MetaKey = ();
    type MetaValue = ();
    type Error = Error;

    fn frame(&mut self, item: Self::Input, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode(dst)
    }

    fn deframe(&mut self, src: &mut BytesMut) -> Result<Self::Output, DeframeError<Self::Error>> {
        if self.decode_state.is_none() {
            self.decode_state = Some(<Self::Output as Decode>::State::default());
        }

        match Self::Output::decode(src, self.decode_state.take().unwrap()) {
            Ok(x) => Ok(x),
            Err(err) => match err {
                DecodeError::Incomplete(s) => {
                    self.decode_state = Some(s);
                    Err(DeframeError::Incomplete)
                }
                DecodeError::Err(e) => Err(DeframeError::Err(e)),
            },
        }
    }

    fn clear(&mut self) {
        self.decode_state = None;
    }

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue) {
        unimplemented!()
    }
}

impl<T> Decode for Response<T>
where
    T: Decode,
    <T as Decode>::Error: 'static,
{
    type Error = Error;
    type State = DecodeState<T::State>;

    fn decode(
        data: &mut BytesMut,
        mut state: Self::State,
    ) -> Result<Self, DecodeError<Self::Error, Self::State>>
    where
        Self: Sized,
    {
        loop {
            match state.state {
                State::Status => state = read_status(data, state)?,
                State::Headers => state = read_header(data, state)?,
                State::Body => {
                    debug!("Decoding Body");
                    return match T::decode(data, state.body_state) {
                        Ok(x) => {
                            debug!("successfull decode");
                            state
                                .builder
                                .body(x)
                                .with_context(|| InvalidResponse)
                                .map_err(DecodeError::Err)
                        }
                        Err(e) => match e {
                            DecodeError::Err(err) => Err(box_error(err))
                                .with_context(|| InvalidBody)
                                .map_err(DecodeError::Err),
                            DecodeError::Incomplete(s) => {
                                state.body_state = s;
                                trace!("incomplete body");
                                Err(DecodeError::Incomplete(state))
                            }
                        },
                    };
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

impl<S, R> Default for Http<S, R>
where
    R: Decode + Send + Sync,
{
    fn default() -> Self {
        Http {
            _send: PhantomData,
            _recv: PhantomData,
            decode_state: None,
        }
    }
}

impl<T> From<Error> for DecodeError<Error, DecodeState<T>> {
    fn from(e: Error) -> Self {
        DecodeError::Err(e)
    }
}

fn read_status<T>(
    data: &mut BytesMut,
    state: DecodeState<T>,
) -> Result<DecodeState<T>, DecodeError<Error, DecodeState<T>>> {
    let (mut raw_status, mut state) = find_eol(data, state)?;

    let http_start = raw_status.windows(5).enumerate().find(|x| x.1 == b"HTTP/");
    if http_start.is_none() {
        return Err(DecodeError::Incomplete(state));
    }
    let http_start = http_start.unwrap().0;

    raw_status.advance(http_start + 5);
    state.builder = state.builder.version(bytes_to_ver(&raw_status[0..3])?);
    raw_status.advance(4); // Skip number and space

    state.builder = state
        .builder
        .status(StatusCode::from_bytes(&raw_status[0..3]).with_context(|| InvalidStatus)?);

    state.state = State::Headers;

    trace!("read http status");
    Ok(state)
}

fn read_header<T>(
    data: &mut BytesMut,
    state: DecodeState<T>,
) -> Result<DecodeState<T>, DecodeError<Error, DecodeState<T>>> {
    let (raw_header, mut state) = find_eol(data, state)?;

    // Reached empty line
    if raw_header.is_empty() {
        state.state = State::Body;
        trace!("finished reading headers");
        return Ok(state);
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

    if name.eq_ignore_ascii_case(b"content-length") {
        state.content_len = Some(
            std::str::from_utf8(&value[1..])
                .with_context(|| InvalidUtf8)
                .with_context(|| InvalidContentLength)?
                .trim()
                .parse()
                .with_context(|| InvalidNumber)
                .with_context(|| InvalidContentLength)?,
        );
    }

    trace!("read http header: {}", String::from_utf8_lossy(name));
    Ok(state)
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

fn find_eol<T>(
    data: &mut BytesMut,
    state: DecodeState<T>,
) -> Result<(BytesMut, DecodeState<T>), DecodeError<Error, DecodeState<T>>> {
    let i = data
        .windows(2)
        .enumerate()
        .find(|x| x.1 == &b"\r\n"[..])
        .map(|x| x.0);
    if i.is_none() {
        return Err(DecodeError::Incomplete(state));
    }

    let raw = data.split_to(i.unwrap());
    data.advance(2); // Skip the newline and return

    Ok((raw, state))
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
