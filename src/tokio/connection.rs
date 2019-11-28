use crate::error::{Error, Send, box_error, Receive};
use crate::error::Connection as ConnectionError;
use crate::{Decode, Encode};
use async_trait::async_trait;
use bytes::BytesMut;
use snafu::ResultExt;

use crate::error::Initiate;
use crate::properties::{Preference, SelectionProperty, TransportProperties};
use std::net::{SocketAddr, Shutdown};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpStream, UdpSocket};
use std::marker::PhantomData;
use std::collections::VecDeque;
use crate::frame::Framer;
use std::process::Output;

const BUFFER_SIZE: usize = 1024;

pub struct Connection<T, F> {
    inner: TokioConnection,
    buffer: BytesMut,
    framer: Option<F>,
    _data: PhantomData<T>
}

enum TokioConnection {
    TCP(TcpStream),
    UDP(UdpSocket),
}

impl TokioConnection {
    async fn send(&mut self, data: &mut BytesMut) -> Result<(), Error> {
        match self {
            TokioConnection::TCP(stream) => stream
                .write_all(data)
                .await
                .map_err(box_error)
                .with_context(|| Send),
            TokioConnection::UDP(socket) => socket
                .send(data)
                .await
                .map(|_| ())
                .map_err(box_error)
                .with_context(|| Send),
        }
    }

    async fn close(self) -> Result<(), Error> {
        match self {
            TokioConnection::TCP(stream) => stream.shutdown(Shutdown::Both)
                .map_err(box_error)
                .with_context(|| ConnectionError),
            TokioConnection::UDP(socket) => Ok(())
        }
    }

    async fn recv(&mut self, data: &mut BytesMut) -> Result<usize, Error> {
        match self {
            TokioConnection::TCP(stream) => stream
                .read(data)
                .await
                .map_err(box_error)
                .with_context(|| Receive),
            TokioConnection::UDP(socket) => socket
                .recv(data)
                .await
                .map_err(box_error)
                .with_context(|| Receive)
        }
    }

    fn abort(self) {
        // Drop self, as underlying types close abort on drop
    }
}

impl<T, F> Connection<T, F>
where
    T: ::std::marker::Send + 'static,
    F: Framer<Input=T, Output=T> + 'static + ::std::marker::Send
{
    pub async fn create(
        addr: SocketAddr,
        props: &TransportProperties,
        framer: Option<F>
    ) -> Result<Box<dyn crate::Connection<T, F>>, Error> {
        let rely: Preference = props.get(SelectionProperty::Reliability);
        let conn = match rely {
            Preference::Require => create_tcp(addr).await?,
            Preference::Prefer | Preference::Ignore => match create_tcp(addr).await {
                Ok(c) => c,
                Err(_) => create_udp(addr).await?,
            },
            Preference::Avoid => match create_udp(addr).await {
                Ok(c) => c,
                Err(_) => create_tcp(addr).await?,
            },
            Preference::Prohibit => create_udp(addr).await?,
        };

        Ok(Box::new(Connection::<T, F> { inner: conn, buffer: BytesMut::new(), framer, _data: PhantomData }))
    }
}

#[async_trait]
impl<T, F> crate::Connection<T, F> for Connection<T, F>
where
    T: ::std::marker::Send + 'static,
    F: Framer + ::std::marker::Send + 'static
{
    async fn send(&mut self, data: T) -> Result<(), Error>
    where
        T: Encode,
    {
        let length = data.size_hint();
        let mut bytes = BytesMut::with_capacity(length.1.unwrap_or_else(|| length.0));
        match &mut self.framer {
            Some(framer) => framer.frame(data, &mut bytes),
            None => data.encode(&mut bytes)
        };
        self.inner.send(&mut bytes).await
    }

    async fn receive(&mut self) -> Result<T, Error>
    where
        T: Decode,
    {
        self.buffer.reserve(BUFFER_SIZE);
        let read = self.inner.recv(&mut self.buffer).await?;
        unimplemented!()
    }

    async fn close(self: Box<Self>) -> Result<(), Error> {
        self.inner.close().await
    }

    fn abort(self: Box<Self>) {
        self.inner.abort()
    }
}

async fn create_tcp(addr: SocketAddr) -> Result<TokioConnection, Error> {
    let stream = TcpStream::connect(addr)
        .await
        .map_err(box_error)
        .with_context(|| Initiate)?;
    Ok(TokioConnection::TCP(stream))
}

async fn create_udp(addr: SocketAddr) -> Result<TokioConnection, Error> {
    let socket = UdpSocket::bind(addr)
        .await
        .map_err(box_error)
        .with_context(|| Initiate)?;
    Ok(TokioConnection::UDP(socket))
}
