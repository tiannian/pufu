//! pufu-core - Core library for pufu

pub use pufu_macros::Codec;

pub mod zc;

/// Trait for types that can be encoded and decoded.
pub trait Codec {}
