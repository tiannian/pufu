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

/// Magic bytes for the binary serialization protocol (see specs/0010-binary-serde.md).
const BINARY_MAGIC: &[u8; 4] = b"svsd";

/// Encoder for building binary payloads conforming to the wire layout in 0010-binary-serde.
/// Accumulates fixed region, variable-entry index (data-relative offsets), and data region.
#[derive(Debug, Default)]
pub struct Encoder {
    /// Bytes for the FixedRegion.
    pub fixed: Vec<u8>,
    /// Data-relative offsets; converted to payload-relative on finalize.
    pub var_idx: Vec<u32>,
    /// Variable-length data region.
    pub data: Vec<u8>,
}

impl Encoder {
    /// Creates an empty Encoder.
    pub fn new() -> Self {
        Self {
            fixed: vec![],
            var_idx: vec![],
            data: vec![],
        }
    }

    /// Pushes the current length of `data` as the next VarEntry offset.
    #[inline]
    pub fn push_offset(&mut self) {
        self.var_idx.push(self.data.len() as u32);
    }

    /// Appends bytes to the fixed region.
    #[inline]
    pub fn push_fixed(&mut self, bytes: &[u8]) {
        self.fixed.extend_from_slice(bytes);
    }

    /// Pushes a data-relative offset onto the variable-entry index.
    #[inline]
    pub fn push_var_idx(&mut self, offset: u32) {
        self.var_idx.push(offset);
    }

    /// Appends bytes to the data region.
    #[inline]
    pub fn push_data(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Writes the full payload (Header + FixedRegion + VarEntry + Data) into `out`.
    /// Uses protocol version 1 and little-endian for multi-byte header fields.
    pub fn finalize(self, out: &mut Vec<u8>) -> Result<(), CodecError> {
        const HEADER_AFTER_MAGIC_VER: usize = 12;
        let var_entry_len = self.var_idx.len().saturating_mul(4);
        let data_region_start = 5u32
            .checked_add((HEADER_AFTER_MAGIC_VER + self.fixed.len() + var_entry_len) as u32)
            .ok_or(CodecError::InvalidLength)?;
        let total_len = HEADER_AFTER_MAGIC_VER + self.fixed.len() + var_entry_len + self.data.len();
        let total_len_u32: u32 = total_len
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;
        let var_entry_offset: u32 = (HEADER_AFTER_MAGIC_VER + self.fixed.len())
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;
        let data_offset: u32 = (HEADER_AFTER_MAGIC_VER + self.fixed.len() + var_entry_len)
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;

        out.reserve(5 + total_len);
        out.extend_from_slice(BINARY_MAGIC);
        out.push(1u8);
        out.extend_from_slice(&total_len_u32.to_le_bytes());
        out.extend_from_slice(&var_entry_offset.to_le_bytes());
        out.extend_from_slice(&data_offset.to_le_bytes());
        out.extend_from_slice(&self.fixed);
        for &off in &self.var_idx {
            let wire = data_region_start
                .checked_add(off)
                .ok_or(CodecError::InvalidLength)?;
            out.extend_from_slice(&wire.to_le_bytes());
        }
        out.extend_from_slice(&self.data);
        Ok(())
    }
}

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
