use crate::properties::TransportProperties;
use crate::tokio::race;
use crate::{Connection, Endpoint, Error};
use async_trait::async_trait;
use std::marker::PhantomData;
use crate::frame::Framer;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preconnection<T, L, R, F> {
    props: TransportProperties,
    local: Option<L>,
    remote: Option<R>,
    framer: Option<F>,
    _data: PhantomData<T>,
}

impl<T, L, R, F> Preconnection<T, L, R, F> {
    pub fn new(props: TransportProperties) -> Self {
        Preconnection {
            props,
            local: None,
            remote: None,
            framer: None,
            _data: PhantomData,
        }
    }
}

#[async_trait]
impl<T, L, R, F> crate::Preconnection<T, L, R, F> for Preconnection<T, L, R, F>
where
    L: Send,
    R: Send,
    F: Send + 'static + Framer
{
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

    async fn initiate(self) -> Result<Box<dyn Connection<T, F>>, Error>
    where
        T: Send + 'static,
        R: Endpoint + Send,
    {
        let remote = self
            .remote
            .expect("cannot initiate a connection without a remote endpoint");
        race::race(remote, self.props, self.framer).await
    }
}
