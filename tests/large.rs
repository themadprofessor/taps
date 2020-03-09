#![allow(unused_must_use)]

use bytes::BytesMut;
use taps::http::Http;
use taps::properties::TransportProperties;
use taps::{Decode, DecodeError, Preconnection};
use toml::Value;
use std::net::SocketAddr;
use std::str::FromStr;
use http::Request;
use http::header::HOST;
use std::ops::Deref;

struct TomlValue(Value);

impl Deref for TomlValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Decode for TomlValue {
    type Error = toml::de::Error;
    type State = ();

    fn decode(
        data: &mut BytesMut,
        state: Self::State,
    ) -> Result<Self, DecodeError<Self::Error, Self::State>>
    where
        Self: Sized,
    {
        eprintln!("{:?}", data);
        toml::from_slice(data).map(TomlValue).map_err(|e| {
            let string = e.to_string();
            if string.contains("unexpected eof") {
                DecodeError::Incomplete(())
            } else {
                DecodeError::Err(e)
            }
        })
    }
}

#[tokio_macros::test]
async fn large_midi() {
    pretty_env_logger::try_init();

    let preconnection = Preconnection::new(
        TransportProperties::default(),
        Http::<String, TomlValue>::default(),
    ).remote_endpoint(SocketAddr::from_str("127.0.0.1:8081").unwrap());

    let mut connection = preconnection.initiate().await.unwrap();
    let mut request = Request::builder()
        .uri("http://127.0.0.1:8081/Cargo.toml")
        .header(HOST, "127.0.0.1")
        .body("".to_string()).unwrap();

    connection.send(request).await.unwrap();

    let response = connection.receive().await.unwrap();
    eprintln!("{:?}", response.body().deref());

    connection.close().await.unwrap();
}
