use crate::tokio::error::Send as SendError;
use crate::tokio::error::{Close, Deframe, Error, Frame, Open, Receive};
use crate::{Decode, Encode};
use async_trait::async_trait;
use bytes::BytesMut;
use snafu::ResultExt;

use crate::error::box_error;
use crate::error::Error as TapsError;
use crate::frame::Framer;
use crate::properties::{Preference, SelectionProperty, TransportProperties};
use log::{debug, trace};
use std::net::{Shutdown, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

const BUFFER_SIZE: usize = 1024;

/// Tokio-based [Connection](../trait.Connection.html) implementation.
#[derive(Debug)]
pub struct Connection<F> {
    inner: TokioConnection,
    buffer: BytesMut,
    framer: F,
}

#[derive(Debug)]
pub(crate) enum TokioConnection {
    TCP(TcpStream),
    UDP(UdpSocket),
}

impl TokioConnection {
    async fn send(&mut self, data: &mut BytesMut) -> Result<(), Error> {
        match self {
            TokioConnection::TCP(stream) => stream
                .write_buf(data)
                .await
                .map(|_| ())
                .with_context(|| SendError),
            TokioConnection::UDP(socket) => socket
                .send(data)
                .await
                .map(|_| ())
                .with_context(|| SendError),
        }
    }

    async fn close(self) -> Result<(), Error> {
        match self {
            TokioConnection::TCP(stream) => stream.shutdown(Shutdown::Both).with_context(|| Close),
            TokioConnection::UDP(_socket) => Ok(()),
        }
    }

    async fn recv(&mut self, data: &mut BytesMut) -> Result<usize, Error> {
        match self {
            TokioConnection::TCP(stream) => stream.read_buf(data).await.with_context(|| Receive),
            TokioConnection::UDP(socket) => socket.recv(data).await.with_context(|| Receive),
        }
    }

    fn abort(self) {
        // Drop self, as underlying types abort on drop
    }
}

impl<F> Connection<F>
where
    F: Framer + 'static + ::std::marker::Send,
    F::Input: ::std::marker::Send,
{
    pub(crate) async fn create(
        addr: SocketAddr,
        props: &TransportProperties,
        framer: F,
    ) -> Result<Box<dyn crate::Connection<F>>, Error> {
        let rely: Preference = props.get(SelectionProperty::Reliability);
        trace!("reliability: {}", rely);
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

        Ok(Box::new(Connection::<F> {
            inner: conn,
            buffer: BytesMut::new(),
            framer,
        }))
    }

    pub(crate) fn from_existing<S>(inner: S, framer: F) -> Box<dyn crate::Connection<F>>
    where
        S: Into<TokioConnection>,
    {
        Box::new(Connection::<F> {
            inner: inner.into(),
            buffer: BytesMut::new(),
            framer,
        })
    }
}

#[async_trait]
impl<F> crate::Connection<F> for Connection<F>
where
    F: Framer + ::std::marker::Send + 'static,
    F::Input: ::std::marker::Send,
{
    async fn send(&mut self, data: F::Input) -> Result<(), TapsError>
    where
        F::Input: Encode,
    {
        let length = data.size_hint();
        trace!("data size hint: {:?}", length);
        let mut bytes = BytesMut::with_capacity(length.1.unwrap_or_else(|| length.0));
        self.framer
            .frame(data, &mut bytes)
            .map_err(box_error)
            .with_context(|| Frame)
            .map_err(box_error)
            .with_context(|| crate::error::Send)?;
        self.inner
            .send(&mut bytes)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Send)
    }

    async fn receive(&mut self) -> Result<F::Output, TapsError>
    where
        F::Output: Decode,
    {
        self.buffer.reserve(BUFFER_SIZE);
        let read = self
            .inner
            .recv(&mut self.buffer)
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Receive)?;
        trace!("bytes read: {}", read);
        self.framer
            .deframe(&mut self.buffer)
            .map(Option::unwrap)
            .map_err(box_error)
            .with_context(|| Deframe)
            .map_err(box_error)
            .with_context(|| crate::error::Receive)
    }

    async fn close(self: Box<Self>) -> Result<(), TapsError> {
        debug!("close connection");
        self.inner
            .close()
            .await
            .map_err(box_error)
            .with_context(|| crate::error::Connection)
    }

    fn abort(self: Box<Self>) {
        debug!("abort connection");
        self.inner.abort()
    }
}

async fn create_tcp(addr: SocketAddr) -> Result<TokioConnection, Error> {
    let stream = TcpStream::connect(addr).await.with_context(|| Open)?;
    trace!("opened tcp");
    Ok(TokioConnection::TCP(stream))
}

async fn create_udp(addr: SocketAddr) -> Result<TokioConnection, Error> {
    let socket = UdpSocket::bind(addr).await.with_context(|| Open)?;
    trace!("opened udp");
    Ok(TokioConnection::UDP(socket))
}

impl From<TcpStream> for TokioConnection {
    fn from(s: TcpStream) -> Self {
        TokioConnection::TCP(s)
    }
}

impl From<UdpSocket> for TokioConnection {
    fn from(s: UdpSocket) -> Self {
        TokioConnection::UDP(s)
    }
}
