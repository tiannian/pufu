//! pufu-core - Core library for pufu

pub mod codec;
pub mod zc;

pub use codec::{Codec, CodecError, Encoder};
pub use pufu_macros::Codec;
