//! Encoder for building binary payloads (no magic or version; see specs/0011-encoder.md).

use crate::codec::CodecError;

/// Encoder for building binary payloads. Accumulates fixed region, variable-entry index
/// (data-relative offsets), and data region. Does not write magic or version.
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

    /// Writes the payload into `out` (appends). Layout: `total_len`, `var_entry_offset`,
    /// `data_offset` (each u32 LE), FixedRegion, VarEntry (payload-relative u32s), Data.
    /// Does not write magic or version.
    pub fn finalize(self, out: &mut Vec<u8>) -> Result<(), CodecError> {
        const HEADER_LEN: usize = 12;
        let var_entry_len = self.var_idx.len().saturating_mul(4);
        let total_len = HEADER_LEN + self.fixed.len() + var_entry_len + self.data.len();
        let total_len_u32: u32 = total_len
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;
        let var_entry_offset: u32 = (HEADER_LEN + self.fixed.len())
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;
        let data_offset: u32 = (HEADER_LEN + self.fixed.len() + var_entry_len)
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;
        let data_region_start = data_offset;

        out.reserve(total_len);
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
