#![allow(unused_must_use)]

use bytes::BytesMut;
use cargo_toml::Manifest;
use http::header::HOST;
use http::Request;
use log::error;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use taps::http::Http;
use taps::properties::TransportProperties;
use taps::{Decode, DecodeError, Preconnection};

#[derive(Debug)]
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

#[tokio_macros::test]
async fn large_midi() {
    pretty_env_logger::init();

    let preconnection = Preconnection::new(
        TransportProperties::default(),
        Http::<String, Cargo>::default(),
    )
    .remote_endpoint(SocketAddr::from_str("127.0.0.1:8081").unwrap());

    let mut connection = preconnection.initiate().await.unwrap();
    let mut request = Request::builder()
        .uri("http://127.0.0.1:8081/Cargo.toml")
        .header(HOST, "127.0.0.1")
        .body("".to_string())
        .unwrap();

    connection.send(request).await.unwrap();

    let response = connection.receive().await.unwrap();
    eprintln!("{:?}", response.body().deref());

    connection.close().await.unwrap();
}
