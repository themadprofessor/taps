use futures::SinkExt;
use std::marker::PhantomData;
use tokio::prelude::*;
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct Connection<C, F> {
    pub(crate) conn: Framed<C, F>,
}

impl<C, F> Connection<C, F>
where
    C: AsyncWrite + Unpin,
    F: Encoder + Unpin,
{
    async fn send(&mut self, data: F::Item) -> Result<(), F::Error> {
        self.conn.send(data).await
    }
}

impl<C, F> Connection<C, F>
where
    C: AsyncRead + Unpin,
    F: Decoder + Unpin,
{
    async fn receive(&mut self) -> Option<Result<F::Item, F::Error>> {
        self.conn.next().await
    }
}
