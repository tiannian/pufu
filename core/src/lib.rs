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

mod config;
pub use config::{Config, ConfigBuilder};

mod data_type;
pub use data_type::*;

/// Endianness used when encoding/decoding fixed-width values.
///
/// **Not serialized**: Endian is never written to or read from the wire. It is only used
/// at encode/decode time to control the byte order of multi-byte header and VarEntry fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    /// Little-endian byte order.
    Little,
    /// Big-endian byte order.
    Big,
    /// Native endianness of the host (encoded as little-endian today).
    Native,
}
