use serde::{Deserializer, Serializer};
use std::marker::PhantomData;

pub struct Connection<'a, SE, DE>
where
    SE: Serializer,
    DE: Deserializer<'a>,
{
    serialize: SE,
    deserialize: DE,
    _phantom: &'a PhantomData<DE>,
}
