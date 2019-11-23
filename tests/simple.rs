use std::net::SocketAddr;
use taps::properties::TransportProperties;
use taps::Endpoint;

#[test]
fn simple() {
    let mut preconnection = ::taps::new_preconnection::<Vec<u8>, SocketAddr, SocketAddr>(
        TransportProperties::default(),
    );

    preconnection.remote_endpoint(SocketAddr::)
}
