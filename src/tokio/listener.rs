use futures::Stream;
use futures::task::{Context, Poll};
use std::pin::Pin;
use std::marker::PhantomData;
use crate::{Framer, Connection};

pub struct Listener<F> {
    limit: Option<usize>,
    framer: PhantomData<F>
}

impl <F> Stream for Listener<F> {
    type Item = Result<Box<dyn Connection<F, Error=crate::error::Error>>, crate::error::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}

impl <F> crate::Listener<F> for Listener<F> where F: Framer + Send + 'static {
    fn connection_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }
}