use http::{HeaderValue, Request, Response, Version};
use std::net::{SocketAddr, SocketAddrV4};
use std::str::FromStr;
use taps::http::Http;
use taps::properties::TransportProperties;
use taps::{Connection, Framer};
use taps::{Endpoint, Preconnection};

#[tokio::test]
async fn simple_http() {
    pretty_env_logger::try_init();

    let mut preconnection =
        ::taps::new_preconnection::<(), SocketAddr, Http<String>>(TransportProperties::default());

    preconnection.remote_endpoint(SocketAddr::from_str("1.1.1.1:80").unwrap());
    preconnection.add_framer(Http::default());

    let mut connection = preconnection.initiate().await.unwrap();
    let mut request = Request::new("".to_string());
    request
        .headers_mut()
        .insert(http::header::HOST, HeaderValue::from_static("1.1.1.1:80"));
    connection.send(request).await.unwrap();

    let response = connection.receive().await.unwrap();
    assert_eq!(response.version(), Version::default());
    assert_eq!(response.status().as_u16(), 301);

    connection.close().await.unwrap();
}

#[tokio::test]
async fn simple_http_dns() {
    pretty_env_logger::try_init();
    let mut preconnection =
        ::taps::new_preconnection::<(), (&str, u16), Http<String>>(TransportProperties::default());

    preconnection.remote_endpoint(("example.com", 80));
    preconnection.add_framer(Http::default());

    let mut connection = preconnection.initiate().await.unwrap();
    let mut request = Request::new("".to_string());
    request.headers_mut().insert(
        http::header::HOST,
        HeaderValue::from_static("example.com:80"),
    );
    connection.send(request).await.unwrap();

    let response = connection.receive().await.unwrap();
    assert_eq!(response.version(), Version::default());

    connection.close().await.unwrap();
}
