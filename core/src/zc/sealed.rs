//! Sealing / disambiguation to prevent `Vec<u8>` from matching fixed-elem `Vec<T>`.

/// Marker trait used to prevent `Vec<u8>` from matching the generic `Vec<T>` fixed-elem vec impl.
/// Also used to exclude other types you don't want treated as fixed-elem sequences.
pub trait NotU8 {}

macro_rules! impl_not_u8_for {
    ($($t:ty),* $(,)?) => {
        $(
            impl NotU8 for $t {}
        )*
    };
}

// All integer types except u8
impl_not_u8_for!(u16, u32, u64, u128, usize);
impl_not_u8_for!(i8, i16, i32, i64, i128, isize);

// NOTE: We deliberately do NOT implement NotU8 for:
// - u8 (obvious)
// - bool (because Vec<bool> is bitpacked and not a contiguous bool array)
// If you later add a custom Bool8 wrapper type that is 1 byte per element, you can implement NotU8 for it.
