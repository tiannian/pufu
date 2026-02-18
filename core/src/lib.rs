//! pufu-core - Core library for pufu

mod encode;
pub use encode::Encode;

mod decode;
pub use decode::Decode;

mod fixed_decode;

mod encoder;
pub use encoder::Encoder;

mod decoder;
pub use decoder::Decoder;

mod codec;
pub use codec::CodecError;

mod data_type;
pub use data_type::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
    Native,
}
