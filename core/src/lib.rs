//! pufu-core - Core library for pufu

mod encode;
pub use encode::FieldEncode;

mod encoder;
pub use encoder::Encoder;

mod sealed;
pub use sealed::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
    Native,
}
