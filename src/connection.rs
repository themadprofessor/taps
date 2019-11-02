use futures::SinkExt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;
use tokio::codec::{Decoder, Encoder, Framed};
use tokio::prelude::*;

pub struct Connection<T, C, F> {
    _phantom: PhantomData<T>,
    conn: Framed<C, F>,
}

impl<T, C, F> Connection<T, C, F>
where
    C: AsyncWrite + Unpin,
    F: Encoder + Unpin,
{
    async fn send(&mut self, data: F::Item) -> Result<(), F::Error> {
        self.conn.send(data).await
    }
}

impl<T, C, F> Connection<T, C, F>
where
    C: AsyncRead + Unpin,
    F: Decoder + Unpin,
{
    async fn receive(&mut self) -> Option<Result<F::Item, F::Error>> {
        self.conn.next().await
    }
}
