use crate::{ElementRank0, Encoder, Endian, NotU8, Result};

pub trait ElementRank1 {
    fn encode(&self, e: &mut Encoder);
}

pub trait NotElementRank0 {}

impl<T: ElementRank0> ElementRank1 for T {
    fn encode(&self, e: &mut Encoder) {
        self.encode(e);
    }
}

impl<T> ElementRank1 for Vec<T>
where
    T: ElementRank0 + NotU8,
{
    fn encode(&self, e: &mut Encoder) {}
}

impl ElementRank1 for Vec<u8> {
    fn encode(&self, e: &mut Encoder) {}
}

impl NotElementRank0 for Vec<u8> {}

impl ElementRank1 for &[u8] {
    fn encode(&self, e: &mut Encoder) {}
}

impl ElementRank1 for &mut [u8] {
    fn encode(&self, e: &mut Encoder) {}
}

impl<T> ElementRank1 for &[T]
where
    T: ElementRank0 + NotU8,
{
    fn encode(&self, e: &mut Encoder) {}
}

impl<T> ElementRank1 for &mut [T]
where
    T: ElementRank0 + NotU8,
{
    fn encode(&self, e: &mut Encoder) {}
}
