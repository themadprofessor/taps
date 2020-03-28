use super::*;
use crate::error::box_error;
use crate::{Decode, DecodeError, DeframeError, Encode, Framer};
use bytes::BytesMut;
use http::request::Builder;
use http::{Method, Request, Response};
use log::debug;
use snafu::ResultExt;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct HttpServer<S, R>
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

impl<S, R> Framer for HttpServer<S, R>
where
    S: Encode + Send + Sync + 'static,
    R: Decode + Send + Sync + 'static,
    <R as Decode>::State: Send + Sync,
{
    type Input = Response<S>;
    type Output = Request<R>;
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

    fn add_metadata(&mut self, _key: Self::MetaKey, _value: Self::MetaValue) {
        unimplemented!()
    }
}

impl<S, R> Clone for HttpServer<S, R>
where
    S: Encode,
    R: Decode + Send + Sync,
{
    fn clone(&self) -> Self {
        HttpServer {
            _send: PhantomData,
            _recv: PhantomData,
            decode_state: None,
        }
    }
}

impl<T> Decode for Request<T>
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
                State::Status => state = read_request(data, state)?,
                State::Headers => state = read_header(data, state)?,
                State::Body => {
                    debug!("Decoding Body");
                    trace!("{:?}", data);
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

impl<T> Encode for Response<T>
where
    T: Encode,
{
    type Error = Error;

    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
        write_response_line(self, data);

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

impl<S, R> Default for HttpServer<S, R>
where
    R: Decode + Send + Sync,
{
    fn default() -> Self {
        HttpServer {
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

fn start_with_method(x: &[u8]) -> bool {
    x.starts_with(Method::GET.as_str().as_bytes())
        || x.starts_with(Method::POST.as_str().as_bytes())
        || x.starts_with(Method::DELETE.as_str().as_bytes())
        || x.starts_with(Method::PUT.as_str().as_bytes())
        || x.starts_with(Method::HEAD.as_str().as_bytes())
        || x.starts_with(Method::OPTIONS.as_str().as_bytes())
        || x.starts_with(Method::CONNECT.as_str().as_bytes())
        || x.starts_with(Method::PATCH.as_str().as_bytes())
        || x.starts_with(Method::TRACE.as_str().as_bytes())
}

fn read_request<T>(
    data: &mut BytesMut,
    state: DecodeState<T>,
) -> Result<DecodeState<T>, DecodeError<Error, DecodeState<T>>> {
    let (mut raw_status, mut state) = find_eol(data, state)?;

    let method_start = raw_status
        .windows(7)
        .enumerate()
        .find(|(_i, x)| start_with_method(x));
    if method_start.is_none() {
        return Err(DecodeError::Incomplete(state));
    }
    let method_start = method_start.unwrap().0;
    raw_status.advance(method_start);

    let method_end = raw_status.iter().enumerate().find(|(_i, x)| **x == b' ');
    if method_end.is_none() {
        return Err(DecodeError::Incomplete(state));
    }
    let method_end = method_end.unwrap().0;
    state.builder = state.builder.method(&raw_status[..method_end]);
    raw_status.advance(method_end + 1);

    let uri_end = raw_status.iter().enumerate().find(|(_i, x)| **x == b' ');
    if uri_end.is_none() {
        return Err(DecodeError::Err(super::Error::InvalidRequestLine));
    }
    let uri_end = uri_end.unwrap().0;
    state.builder = state.builder.uri(&raw_status[..uri_end]);
    raw_status.advance(uri_end + 1 + 5);
    debug!("{:?}", raw_status);

    state.builder = state.builder.version(bytes_to_ver(&raw_status[0..3])?);
    raw_status.advance(3);

    state.state = State::Headers;

    trace!("read http status");
    Ok(state)
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
