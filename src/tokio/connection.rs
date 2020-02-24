use crate::tokio::error::Send as SendError;
use crate::tokio::error::{Close, Deframe, Error, Frame, Open, Receive};
use crate::{Decode, Encode};
use async_trait::async_trait;
use bytes::BytesMut;
use snafu::ResultExt;

use crate::error::box_error;
use crate::error::Error as TapsError;
use crate::properties::{Preference, SelectionProperty, TransportProperties};
use crate::Framer;
use log::{debug, trace};
use std::marker::PhantomData;
use std::net::{Shutdown, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

const BUFFER_SIZE: usize = 1024;

/// Tokio-based [Connection](../trait.Connection.html) implementation.
#[derive(Debug)]
pub struct Connection<F, S, R>
where
    F: Framer<S, R>,
{
    inner: TokioConnection,
    buffer: BytesMut,
    framer: F,
    local: SocketAddr,
    remote: SocketAddr,
    _send: PhantomData<S>,
    _recv: PhantomData<R>,
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

    fn local(&self) -> ::std::io::Result<SocketAddr> {
        match self {
            TokioConnection::TCP(stream) => stream.local_addr(),
            TokioConnection::UDP(socket) => socket.local_addr(),
        }
    }
}

impl<F, S, R> Connection<F, S, R>
where
    F: Framer<S, R>,
    S: Send + 'static,
    R: Send + 'static,
{
    pub(crate) async fn create(
        addr: SocketAddr,
        props: &TransportProperties,
        framer: F,
    ) -> Result<Box<dyn crate::Connection<F, S, R>>, Error> {
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

        let local = conn.local().with_context(|| Open)?;
        Ok(Box::new(Connection::<F, S, R> {
            inner: conn,
            buffer: BytesMut::new(),
            framer,
            remote: addr,
            local,
            _recv: PhantomData,
            _send: PhantomData,
        }))
    }

    pub(crate) fn from_existing<I>(
        inner: I,
        framer: F,
        remote: SocketAddr,
    ) -> Result<Box<dyn crate::Connection<F, S, R>>, Error>
    where
        I: Into<TokioConnection>,
    {
        let conn = inner.into();
        let local = conn.local().with_context(|| Open)?;
        Ok(Box::new(Connection::<F, S, R> {
            inner: conn,
            buffer: BytesMut::new(),
            framer,
            remote,
            local,
            _send: PhantomData,
            _recv: PhantomData,
        }))
    }
}

#[async_trait]
impl<F, S, R> crate::Connection<F, S, R> for Connection<F, S, R>
where
    F: Framer<S, R>,
    S: Send,
    R: Send,
{
    async fn send(&mut self, data: S) -> Result<(), TapsError>
    where
        S: Encode,
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

    async fn receive(&mut self) -> Result<R, TapsError>
    where
        R: Decode,
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

    fn remote_endpoint(&self) -> SocketAddr {
        self.remote
    }

    fn local_endpoint(&self) -> SocketAddr {
        self.local
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
