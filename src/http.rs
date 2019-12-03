use crate::error::Error;
use crate::frame::Framer;
use crate::{Decode, Encode};
use bytes::BytesMut;
use http::header::HeaderName;
use http::{HeaderMap, HeaderValue, Request, Response};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Http<T> {
    headers: HeaderMap,
    _data: PhantomData<T>,
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
        data.extend_from_slice(b" HTTP/1.0\r\n");

        for (header, value) in req.headers().iter() {
            if !req.headers().contains_key(header) {
                data.extend_from_slice(header.as_ref());
                data.extend_from_slice(&[b':']);
                data.extend_from_slice(value.as_bytes());
                data.extend_from_slice(b"\r\n");
            }
        }

        req.body().encode(data)
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
        unimplemented!()
    }

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue) {
        self.headers.insert(key, value);
    }
}
