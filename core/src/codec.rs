//! Codec trait and error type for zero-copy encode/decode.

use std::fmt;

/// Error type for codec operations (validation or decode failure).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    /// Buffer length or layout is invalid.
    InvalidLength,
    /// Data failed validation (e.g. checksum, magic, or structural check).
    ValidationFailed,
    /// Custom message for diagnostics.
    Message(String),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::InvalidLength => write!(f, "invalid length"),
            CodecError::ValidationFailed => write!(f, "validation failed"),
            CodecError::Message(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for CodecError {}

/// Trait for types that can be encoded and decoded with an optional zero-copy view.
pub trait Codec: Sized {
    /// Zero-copy view type.
    type View<'a>: Sized
    where
        Self: 'a;

    /// Encode into the given byte vector (appends).
    fn encode(&self, buf: &mut Vec<u8>);

    /// Decode into a zero-copy view.
    fn decode<'a>(buf: &'a [u8]) -> Result<Self::View<'a>, CodecError>;

    /// Optionally validate the buffer without constructing the view.
    fn validate(buf: &[u8]) -> Result<(), CodecError>;
}
