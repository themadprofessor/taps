use crate::frame::Framer;
use bytes::BytesMut;
use crate::error::Error;
use http::{HeaderMap, Request, Response, HeaderValue};
use std::marker::PhantomData;
use crate::{Encode, Decode};
use http::header::HeaderName;

#[derive(Debug)]
pub struct Http<T> {
    headers: HeaderMap,
    _data: PhantomData<T>
}

impl <T> Framer for Http<T> where T: Encode + Decode {
    type Input = Request<T>;
    type Output = Response<T>;
    type MetaKey = HeaderName;
    type MetaValue = HeaderValue;

    fn frame(&mut self, item: Self::Input, dst: &mut BytesMut) -> Result<(), Error> {
        dst.extend_from_slice(item.method().as_str().as_bytes());
        dst.extend_from_slice(&[b' ']);
        dst.extend_from_slice(item.uri().path_and_query().map(|p| p.as_str()).unwrap_or_else(|| "/").as_bytes());
        dst.extend_from_slice(b" HTTP/1.0\r\n");

        for (header, value) in self.headers.iter() {
            if !item.headers().contains_key(header) {
                dst.extend_from_slice(header.as_ref());
                dst.extend_from_slice(&[b':']);
                dst.extend_from_slice(value.as_bytes());
                dst.extend_from_slice(b"\r\n");
            }
        }

        for (header, value) in item.headers().iter() {
            dst.extend_from_slice(header.as_ref());
            dst.extend_from_slice(&[b':']);
            dst.extend_from_slice(value.as_bytes());
            dst.extend_from_slice(b"\r\n");
        }
        dst.extend_from_slice(b"\r\n");

        item.body().encode(dst)
    }

    fn deframe(&mut self, src: &mut BytesMut) -> Result<Option<Self::Output>, Error> {
        unimplemented!()
    }

    fn add_metadata(&mut self, key: Self::MetaKey, value: Self::MetaValue) {
        self.headers.insert(key, value);
    }
}