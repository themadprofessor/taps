use crate::error::{Error, Send};
use crate::{Decode, Encode};
use async_trait::async_trait;
use bytes::BytesMut;
use futures::SinkExt;
use snafu::ResultExt;
use std::marker::Send as SendTrait;
use tokio::io::AsyncWriteExt;
use tokio::prelude::*;

pub struct Stream<C> {
    inner: C,
}

impl <C> Stream<C> {
    pub(crate) fn new(inner: C) -> Self {
        Stream { inner }
    }
}

#[async_trait]
impl<T, C> crate::Connection<T> for Stream<C>
where
    T: SendTrait + 'static,
{
    async fn send(&mut self, data: T) -> Result<(), Error>
    where
        T: Encode,
        C: AsyncWrite + Unpin,
    {
        let data_len = data.size_hint();
        let mut bytes = BytesMut::with_capacity(data_len.1.unwrap_or(data_len.0));
        data.encode(&mut bytes)?;
        self.inner
            .write(&bytes)
            .await
            .map(|_| ())
            .map_err(Into::into)
            .with_context(|| Send)
    }

    async fn receive(&mut self) -> Result<T, Error>
    where
        T: Decode,
        C: AsyncRead,
    {
        unimplemented!()
    }

    async fn close(self) -> Result<(), Error> {
        unimplemented!()
    }

    fn abort(self) {
        unimplemented!()
    }
}
