pub trait NotU8 {}

macro_rules! impl_not_u8 {
    ($($ty:ty),*) => {
        $(impl NotU8 for $ty {})*
    };
}

impl_not_u8!(u16, u32, u64, u128, i16, i32, i64, i128, f32, f64);

mod rank0;
pub use rank0::*;

mod rank1;
pub use rank1::*;
