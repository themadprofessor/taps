use crate::error::Error;
use crate::{Decode, Encode};
use async_trait::async_trait;
use futures::SinkExt;
use tokio::prelude::*;
use tokio::io::AsyncWriteExt;
use bytes::BytesMut;
use snafu::ResultExt;

pub struct Stream<C> {
    inner: C
}

#[async_trait]
impl<T, C> crate::Connection<T> for Stream<C>
where
    T: Send + 'static,
{
    async fn send(&mut self, data: T) -> Result<(), Error>
    where
        T: Encode,
        C: AsyncWrite + Unpin
    {
        let data_len = data.size_hint();
        let mut bytes = BytesMut::with_capacity(data_len.1.unwrap_or(data_len.0));
        data.encode(&mut bytes)?;
        self.inner.write(&bytes).await.map(|_| ()).with_context()
    }

    async fn receive(&mut self) -> Result<T, Error>
    where
        T: Decode,
        C: AsyncRead
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
