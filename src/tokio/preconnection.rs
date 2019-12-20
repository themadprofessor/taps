use crate::frame::Framer;
use crate::properties::TransportProperties;
use crate::resolve::Endpoint;
use crate::tokio::error::Error;
use crate::tokio::race;
use crate::Connection;
use async_trait::async_trait;

/// Tokio-based [Preconnection](../trait.Preconnection.html) implementation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preconnection<L, R, F> {
    props: TransportProperties,
    local: Option<L>,
    remote: Option<R>,
    framer: Option<F>,
}

impl<L, R, F> Preconnection<L, R, F> {
    pub fn new(props: TransportProperties) -> Self {
        Preconnection {
            props,
            local: None,
            remote: None,
            framer: None,
        }
    }
}

#[async_trait]
impl<L, R, F> crate::Preconnection<L, R, F> for Preconnection<L, R, F>
where
    L: Send,
    R: Send,
    F: Send + Sync + 'static + Framer + Clone,
{
    type Error = Error;

    fn local_endpoint(&mut self, local: L)
    where
        L: Endpoint,
    {
        self.local = Some(local)
    }

    fn remote_endpoint(&mut self, remote: R)
    where
        R: Endpoint,
    {
        self.remote = Some(remote)
    }

    fn transport_properties(&self) -> &TransportProperties {
        &self.props
    }

    fn transport_properties_mut(&mut self) -> &mut TransportProperties {
        &mut self.props
    }

    fn add_framer(&mut self, framer: F) {
        self.framer = Some(framer)
    }

    async fn initiate(self) -> Result<Box<dyn Connection<F, Error = Self::Error> + Send>, Self::Error>
    where
        R: Endpoint + Send,
        <R as Endpoint>::Error: 'static,
        F::Input: ::std::marker::Send,
    {
        let remote = self
            .remote
            .expect("cannot initiate a connection without a remote endpoint");
        race::race(remote, self.props, self.framer.unwrap()).await
    }
}
