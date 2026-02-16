//! ZcFixed implementations for primitives and fixed-size arrays.

use core::mem::{align_of, size_of};

use super::sealed::NotU8;
use super::traits::ZcFixed;

// Unsigned ints
impl ZcFixed for u8 {
    const SIZE: usize = 1;
    const ALIGN: usize = align_of::<u8>();
}
impl ZcFixed for u16 {
    const SIZE: usize = 2;
    const ALIGN: usize = align_of::<u16>();
}
impl ZcFixed for u32 {
    const SIZE: usize = 4;
    const ALIGN: usize = align_of::<u32>();
}
impl ZcFixed for u64 {
    const SIZE: usize = 8;
    const ALIGN: usize = align_of::<u64>();
}
impl ZcFixed for u128 {
    const SIZE: usize = 16;
    const ALIGN: usize = align_of::<u128>();
}
impl ZcFixed for usize {
    const SIZE: usize = size_of::<usize>();
    const ALIGN: usize = align_of::<usize>();
}

// Signed ints
impl ZcFixed for i8 {
    const SIZE: usize = 1;
    const ALIGN: usize = align_of::<i8>();
}
impl ZcFixed for i16 {
    const SIZE: usize = 2;
    const ALIGN: usize = align_of::<i16>();
}
impl ZcFixed for i32 {
    const SIZE: usize = 4;
    const ALIGN: usize = align_of::<i32>();
}
impl ZcFixed for i64 {
    const SIZE: usize = 8;
    const ALIGN: usize = align_of::<i64>();
}
impl ZcFixed for i128 {
    const SIZE: usize = 16;
    const ALIGN: usize = align_of::<i128>();
}
impl ZcFixed for isize {
    const SIZE: usize = size_of::<isize>();
    const ALIGN: usize = align_of::<isize>();
}

// bool as fixed (1 byte in memory representation for a standalone bool value)
impl ZcFixed for bool {
    const SIZE: usize = size_of::<bool>();
    const ALIGN: usize = align_of::<bool>();
}

// Fixed-size arrays: [T; N] is fixed if T is fixed.
impl<T: ZcFixed, const N: usize> ZcFixed for [T; N] {
    const SIZE: usize = T::SIZE * N;
    const ALIGN: usize = T::ALIGN;
}

// We also want Vec<[u8;32]> and Vec<[u64;4]> etc to be allowed as fixed-elem vec,
// so we mark *arrays* as NotU8 (the element type itself isn't u8; it's [u8;N]).
impl<T: ZcFixed, const N: usize> NotU8 for [T; N] {}
