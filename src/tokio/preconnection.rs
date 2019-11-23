use crate::properties::TransportProperties;
use crate::tokio::race;
use crate::{Connection, Endpoint, Error};
use async_trait::async_trait;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preconnection<T, L, R> {
    props: TransportProperties,
    local: Option<L>,
    remote: Option<R>,
    _data: PhantomData<T>,
}

impl<T, L, R> Preconnection<T, L, R> {
    pub fn new(props: TransportProperties) -> Self {
        Preconnection {
            props,
            local: None,
            remote: None,
            _data: PhantomData,
        }
    }
}

#[async_trait]
impl<T, L, R> crate::Preconnection<T, L, R> for Preconnection<T, L, R>
where
    L: Send,
    R: Send,
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

    async fn initiate(self) -> Result<Box<dyn Connection<T>>, Error>
    where
        T: Send + 'static,
        R: Endpoint + Send,
    {
        let remote = self
            .remote
            .expect("cannot initiate a connection without a remote endpoint");
        race::race(remote, self.props).await
    }
}
