use crate::Endpoint;
use async_trait::async_trait;
use futures::compat::Future01CompatExt;
use snafu::ResultExt;
use std::net::SocketAddr;

mod error;
pub mod preconnection;

#[async_trait]
impl Endpoint for SocketAddr {
    async fn resolve(self) -> Result<Vec<SocketAddr>, crate::Error> {
        Ok(vec![self])
    }
}
