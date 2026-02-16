//! Minimal trait taxonomy for Zc layout:
//!
//! - **ZcFixed**: goes into Fixed region (compile-time size/alignment).
//! - **ZcVar1**: one "variable-length segment" that gets exactly ONE offset in VarIdx.
//!   Segment length = next_offset - this_offset.
//!   The segment can be:
//!   - bytes-like (`ELEM_SIZE = None`): `Vec<u8>`, `String`
//!   - fixed-elem sequence (`ELEM_SIZE = Some(sz)`): `Vec<u64>`, `Vec<[u8;32]>`, `Vec<[u64;4]>`, ...
//! - **ZcVar2**: "second-level variable array": `Vec<Inner>` where `Inner` itself is a ZcVar1 segment,
//!   each item contributes one offset in VarIdx; item length derived by adjacent offsets.
//!
//! NOTE:
//! - `Vec<bool>` is NOT supported as fixed-elem vec: Rust's `Vec<bool>` is bitpacked, not `&[bool]`.
//! - We intentionally disambiguate `Vec<u8>` as bytes-like and prevent it from matching `Vec<T: ZcFixed>`.

mod fixed;
mod sealed;
mod traits;
mod var1;

pub use traits::{ZcFixed, ZcVar1, ZcVar2};
