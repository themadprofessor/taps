#![allow(unused_must_use)]

use bytes::BytesMut;
use cargo_toml::Manifest;
use futures::StreamExt;
use http::header::HOST;
use http::{Request, Response};
use log::{error, info};
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use taps::http::{HttpClient, HttpServer};
use taps::properties::TransportProperties;
use taps::{Decode, DecodeError, Preconnection, Encode};

#[derive(Debug, Clone)]
struct Cargo(Manifest);

impl Deref for Cargo {
    type Target = Manifest;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Decode for Cargo {
    type Error = cargo_toml::Error;
    type State = ();

    fn decode(
        data: &mut BytesMut,
        _state: Self::State,
    ) -> Result<Self, DecodeError<Self::Error, Self::State>>
    where
        Self: Sized,
    {
        Manifest::from_slice(&data).map(Cargo).map_err(|e| {
            error!("{}", e);
            DecodeError::Incomplete(())
        })
    }
}

impl Encode for Cargo {
    type Error = toml::ser::Error;

    fn encode(&self, data: &mut BytesMut) -> Result<(), Self::Error> {
        toml::to_vec(self.deref()).map(|x| {
            data.extend_from_slice(&x);
        })
    }
}

#[ignore]
#[tokio_macros::test]
async fn large_cargo() {
    pretty_env_logger::init();

    let preconnection = Preconnection::new(
        TransportProperties::default(),
        HttpClient::<(), Cargo>::default(),
    )
    .remote_endpoint(SocketAddr::from_str("127.0.0.1:8081").unwrap());

    let mut connection = preconnection.initiate().await.unwrap();
    let request = Request::builder()
        .uri("http://127.0.0.1:8081/Cargo.toml")
        .header(HOST, "127.0.0.1")
        .body(())
        .unwrap();

    connection.send(request).await.unwrap();

    let response = connection.receive().await.unwrap();
    info!("{:?}", response.body().deref());

    connection.close().await.unwrap();
}

#[ignore]
#[tokio_macros::test]
async fn listen_cargo() {
    pretty_env_logger::init();

    let preconnection = Preconnection::new(
        TransportProperties::default(),
        HttpServer::<(), Cargo>::default(),
    )
    .local_endpoint(SocketAddr::from_str("127.0.0.1:8081").unwrap());

    let mut listener = preconnection.listen().await.unwrap();
    if let Some(conn) = listener.next().await {
        let mut conn = conn.unwrap();
        let response = conn.receive().await.unwrap();
        info!("{:?}", response.body().deref());

        conn.send(Response::builder().body(()).unwrap()).await.unwrap();
    }
}
