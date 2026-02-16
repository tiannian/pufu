//! ZcVar1 implementations for bytes-like and fixed-element segments.

use super::sealed::NotU8;
use super::traits::{ZcFixed, ZcVar1};

// Bytes-like segments
impl ZcVar1 for Vec<u8> {
    const ELEM_SIZE: Option<usize> = None;
}

// Treat String as bytes-like UTF-8 segment (view layer may expose &[u8] or &str with UTF-8 check)
impl ZcVar1 for String {
    const ELEM_SIZE: Option<usize> = None;
}

// Fixed-element segments: Vec<T> where T is fixed-size AND not u8 (to avoid overlap with Vec<u8>).
// This covers Vec<u16>, Vec<u64>, Vec<i32>, Vec<[u8;32]>, Vec<[u64;4]>, Vec<MyHash>, etc.
impl<T: ZcFixed + NotU8> ZcVar1 for Vec<T> {
    const ELEM_SIZE: Option<usize> = Some(T::SIZE);
}

// Slice views: bytes-like
impl ZcVar1 for &[u8] {
    const ELEM_SIZE: Option<usize> = None;
}
impl ZcVar1 for &mut [u8] {
    const ELEM_SIZE: Option<usize> = None;
}

// Slice views: fixed-element segments
impl<T: ZcFixed + NotU8> ZcVar1 for &[T] {
    const ELEM_SIZE: Option<usize> = Some(T::SIZE);
}
impl<T: ZcFixed + NotU8> ZcVar1 for &mut [T] {
    const ELEM_SIZE: Option<usize> = Some(T::SIZE);
}
