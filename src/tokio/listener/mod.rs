use crate::properties::{Preference, SelectionProperty, TransportProperties};
use crate::tokio::error::Error;
use crate::{Framer, Listener, Connection, MakeSimilar};
use snafu::ResultExt;
use std::net::SocketAddr;

mod tcp;

pub(crate) async fn open_listener<F>(
    addr: SocketAddr,
    remote: Option<SocketAddr>,
    props: &TransportProperties,
    framer: F,
) -> Result<Box<dyn Listener<F, Item=Result<Box<dyn Connection<F>>, crate::error::Error>>>, Error>
where
    F: Framer + MakeSimilar + Unpin,
{
    let rely = props.get(SelectionProperty::Reliability);
    Ok(match rely {
        Preference::Require => create_tcp(addr, framer, remote).await?,
        Preference::Prefer | Preference::Ignore => {
            let clone = framer.make_similar();
            match create_tcp(addr, framer, remote).await {
                Ok(c) => c,
                Err(_) => create_udp(addr, clone, remote).await?,
            }
        },
        Preference::Avoid => {
            let clone = framer.make_similar();
            match create_udp(addr, framer, remote).await {
                Ok(c) => c,
                Err(_) => {
                    create_tcp(addr, clone, remote).await?
                },
            }
        },
        Preference::Prohibit => create_udp(addr, framer, remote).await?,
    })
}

async fn create_tcp<F>(addr: SocketAddr, framer: F, remote: Option<SocketAddr>) -> Result<Box<dyn Listener<F>>, Error>
where
    F: Framer + MakeSimilar + Unpin,
{
    tcp::Listener::create(addr, remote, framer).await
}

async fn create_udp<F>(_addr: SocketAddr, _framer: F, _remote: Option<SocketAddr>) -> Result<Box<dyn Listener<F>>, Error>
where
    F: Framer + MakeSimilar + Unpin,
{
    unimplemented!()
}
