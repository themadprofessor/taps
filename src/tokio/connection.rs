use std::net::{Shutdown, SocketAddr};

use bytes::BytesMut;
use log::{debug, trace};
use snafu::ResultExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

use async_trait::async_trait;

use crate::codec::DeframeError;
use crate::error::box_error;
use crate::error::Error as TapsError;
use crate::properties::{Preference, SelectionProperty, TransportProperties};
use crate::tokio::error::Send as SendError;
use crate::tokio::error::{Close, Deframe, Error, Frame, Open, Receive};
use crate::Encode;
use crate::Framer;
use std::sync::Arc;

const BUFFER_SIZE: usize = 1024;

/// Tokio-based [Connection](../trait.Connection.html) implementation.
#[derive(Debug)]
pub struct Connection<F>
where
    F: Framer,
{
    inner: TokioConnection,
    buffer: BytesMut,
    framer: F,
    local: SocketAddr,
    remote: SocketAddr,
}

pub(crate) struct Connecting {
    inner: TokioConnection,
    local: SocketAddr,
    remote: SocketAddr,
}

#[derive(Debug)]
pub(crate) enum TokioConnection {
    TCP(TcpStream),
    UDP(UdpSocket),
}

impl Connecting {
    pub(crate) async fn create(
        addr: SocketAddr,
        props: Arc<TransportProperties>,
    ) -> Result<Connecting, Error> {
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
        Ok(Connecting {
            remote: addr,
            local,
            inner: conn,
        })
    }

    pub(crate) fn framer<F>(self, framer: F) -> Box<dyn crate::Connection<F>>
    where
        F: Framer,
    {
        Box::new(Connection {
            inner: self.inner,
            remote: self.remote,
            local: self.local,
            framer,
            buffer: BytesMut::new(),
        })
    }
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

impl<F> Connection<F>
where
    F: Framer,
{
    pub(crate) async fn create(
        addr: SocketAddr,
        props: Arc<TransportProperties>,
        framer: F,
    ) -> Result<Box<dyn crate::Connection<F>>, Error> {
        Ok(Connecting::create(addr, props).await?.framer(framer))
    }

    pub(crate) fn from_existing<I>(
        inner: I,
        framer: F,
        remote: SocketAddr,
    ) -> Result<Box<dyn crate::Connection<F>>, Error>
    where
        I: Into<TokioConnection>,
    {
        let conn = inner.into();
        let local = conn.local().with_context(|| Open)?;
        Ok(Box::new(Connection::<F> {
            inner: conn,
            buffer: BytesMut::new(),
            framer,
            remote,
            local,
        }))
    }
}

#[async_trait]
impl<F> crate::Connection<F> for Connection<F>
where
    F: Framer,
{
    async fn send(&mut self, data: F::Input) -> Result<(), TapsError> {
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

    async fn receive(&mut self) -> Result<F::Output, TapsError> {
        loop {
            self.buffer.reserve(BUFFER_SIZE);
            let mut buffer = self.buffer.split_off(self.buffer.len());
            let read = self
                .inner
                .recv(&mut buffer)
                .await
                .map_err(box_error)
                .with_context(|| crate::error::Receive)?;

            if read == 0 && buffer.is_empty() {
                return Err(box_error(Error::Receive {
                    source: tokio::io::ErrorKind::UnexpectedEof.into(),
                }))
                .with_context(|| Deframe)
                .map_err(box_error)
                .with_context(|| crate::error::Receive);
            }

            self.buffer.unsplit(buffer);

            debug!("bytes read: {}", read);
            trace!("{:?}", self.buffer);
            match self.framer.deframe(&mut self.buffer) {
                Ok(x) => {
                    self.framer.clear();
                    self.buffer.clear();
                    return Ok(x);
                }
                Err(e) => match e {
                    DeframeError::Incomplete => continue,
                    DeframeError::Err(err) => {
                        self.framer.clear();
                        return Err(box_error(err))
                            .with_context(|| Deframe)
                            .map_err(box_error)
                            .with_context(|| crate::error::Receive);
                    }
                },
            };
        }
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
