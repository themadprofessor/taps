use http::Request;
use std::net::{SocketAddr, SocketAddrV4};
use std::str::FromStr;
use taps::http::Http;
use taps::properties::TransportProperties;
use taps::{Endpoint, Preconnection};

#[tokio::test]
async fn simple_tcp() {
    let mut preconnection =
        ::taps::new_preconnection::<Vec<u8>, SocketAddr, SocketAddr, Http<String>>(
            TransportProperties::default(),
        );

    preconnection.remote_endpoint(SocketAddr::from_str("1.1.1.1:80").unwrap());
    let mut connection = preconnection.initiate().await.unwrap();
    connection.send(Request::new("foo".to_string()));
    connection.close().await.unwrap();
}
