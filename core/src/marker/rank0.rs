use crate::{Encoder, Endian, Error, NotU8, Result};

pub trait ElementRank0: Sized {
    fn encode(&self, e: &mut Encoder);
}

impl ElementRank0 for u8 {
    fn encode(&self, e: &mut Encoder) {
        e.fixed.push(*self);
    }
}

macro_rules! impl_element_rank0 {
    ($($ty:ty),*) => {
        $(impl ElementRank0 for $ty {
            fn encode(&self, e: &mut Encoder) {
                let self_bytes = match e.config.endian {
                    Endian::Little => self.to_le_bytes(),
                    Endian::Big => self.to_be_bytes(),
                    Endian::Native => self.to_ne_bytes(),
                };
                e.fixed.extend_from_slice(&self_bytes);
            }
        })*
    };
}

impl_element_rank0!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

impl<const N: usize> ElementRank0 for [u8; N] {
    fn encode(&self, e: &mut Encoder) {
        e.fixed.extend_from_slice(self);
    }
}

impl<T, const N: usize> ElementRank0 for [T; N]
where
    T: ElementRank0 + NotU8,
{
    fn encode(&self, e: &mut Encoder) {
        for item in self {
            item.encode(e);
        }
    }
}
