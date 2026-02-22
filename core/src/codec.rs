//! Codec error types for pufu payloads.

/// Errors returned by encoding/decoding operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    /// Input lengths or offsets do not match the expected layout.
    InvalidLength,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::InvalidLength => write!(f, "invalid length"),
        }
    }
}

impl std::error::Error for CodecError {}
