mod decode;
mod encode;
mod error;
mod frame;
pub use decode::Decode;
pub use encode::Encode;
pub use error::*;
pub use frame::Framer;

pub trait MakeSimilar{
    fn make_similar(&self) -> Self;
}
