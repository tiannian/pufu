//! Decoder for reading binary payloads (no magic or version; see specs/0012-decoder.md).

use crate::codec::CodecError;

/// Decoder for reading binary payloads produced by `Encoder`.
///
/// The layout is:
/// - `total_len` (u32, LE)
/// - `var_entry_offset` (u32, LE)
/// - `data_offset` (u32, LE)
/// - FixedRegion bytes
/// - VarEntry region (u32 LE offsets, relative to Data region start)
/// - Data region bytes
///
/// This decoder operates on a view into the buffer (`buf`) whose first
/// bytes are the payload header written by `Encoder::finalize`.
#[derive(Debug, Clone, Copy)]
pub struct Decoder<'a> {
    /// Complete buffer that contains the payload.
    pub buf: &'a [u8],
    /// Total payload length in bytes, as read from the header.
    pub total_len: u32,
    /// Offset (relative to the start of this payload) where the VarEntry region starts.
    pub var_idx_offset: u32,
    /// Offset (relative to the start of this payload) where the Data region starts.
    pub data_offset: u32,
    /// Current cursor in the FixedRegion (relative to the FixedRegion start).
    pub fixed_cursor: u32,
    /// Current cursor in the VarEntry region (relative to `var_idx_offset`).
    pub var_cursor: u32,
}

impl<'a> Decoder<'a> {
    const HEADER_LEN: u32 = 12;

    /// Creates a new Decoder by parsing the header from `buf`.
    ///
    /// Expects the first 12 bytes of `buf` to be:
    /// - `total_len` (u32 LE)
    /// - `var_entry_offset` (u32 LE)
    /// - `data_offset` (u32 LE)
    ///
    /// Validates that these values describe a layout fully contained within `buf`.
    pub fn new(buf: &'a [u8]) -> Result<Self, CodecError> {
        if buf.len() < Self::HEADER_LEN as usize {
            return Err(CodecError::InvalidLength);
        }

        let total_len = {
            let bytes: [u8; 4] = buf[0..4]
                .try_into()
                .map_err(|_| CodecError::InvalidLength)?;
            u32::from_le_bytes(bytes)
        };
        let var_idx_offset = {
            let bytes: [u8; 4] = buf[4..8]
                .try_into()
                .map_err(|_| CodecError::InvalidLength)?;
            u32::from_le_bytes(bytes)
        };
        let data_offset = {
            let bytes: [u8; 4] = buf[8..12]
                .try_into()
                .map_err(|_| CodecError::InvalidLength)?;
            u32::from_le_bytes(bytes)
        };

        let total_len_usize: usize = total_len
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;
        if total_len_usize > buf.len() {
            return Err(CodecError::InvalidLength);
        }

        // Basic structural checks.
        if var_idx_offset < Self::HEADER_LEN {
            return Err(CodecError::InvalidLength);
        }
        if data_offset < var_idx_offset {
            return Err(CodecError::InvalidLength);
        }
        if data_offset > total_len {
            return Err(CodecError::InvalidLength);
        }

        Ok(Self {
            buf,
            total_len,
            var_idx_offset,
            data_offset,
            fixed_cursor: 0,
            var_cursor: 0,
        })
    }

    /// Returns the number of variable-length entries in the VarEntry region.
    ///
    /// This is `(data_offset - var_idx_offset) / 4`.
    pub fn var_count(&self) -> Result<u32, CodecError> {
        let span = self
            .data_offset
            .checked_sub(self.var_idx_offset)
            .ok_or(CodecError::InvalidLength)?;
        if span % 4 != 0 {
            return Err(CodecError::InvalidLength);
        }
        Ok(span / 4)
    }

    /// Internal helper to compute the FixedRegion length in bytes.
    fn fixed_region_len(&self) -> Result<u32, CodecError> {
        self.var_idx_offset
            .checked_sub(Self::HEADER_LEN)
            .ok_or(CodecError::InvalidLength)
    }

    /// Reads the next `len` bytes from the FixedRegion, advancing `fixed_cursor`.
    pub fn next_fixed_bytes(&mut self, len: u32) -> Result<&'a [u8], CodecError> {
        let fixed_len = self.fixed_region_len()?;

        let new_cursor = self
            .fixed_cursor
            .checked_add(len)
            .ok_or(CodecError::InvalidLength)?;
        if new_cursor > fixed_len {
            return Err(CodecError::InvalidLength);
        }

        let start_abs = Self::HEADER_LEN
            .checked_add(self.fixed_cursor)
            .ok_or(CodecError::InvalidLength)?;
        let end_abs = start_abs
            .checked_add(len)
            .ok_or(CodecError::InvalidLength)?;

        // Must stay within the payload.
        if end_abs > self.total_len {
            return Err(CodecError::InvalidLength);
        }

        let start = usize::try_from(start_abs).map_err(|_| CodecError::InvalidLength)?;
        let end = usize::try_from(end_abs).map_err(|_| CodecError::InvalidLength)?;

        if end > self.buf.len() {
            return Err(CodecError::InvalidLength);
        }

        self.fixed_cursor = new_cursor;
        Ok(&self.buf[start..end])
    }

    /// Reads the `idx`-th variable-length value using VarEntry offsets.
    ///
    /// - Reads the `idx`-th `u32` from the VarEntry region as an absolute payload
    ///   offset into the Data region.
    /// - For all but the last entry, also reads the `(idx + 1)`-th `u32` offset
    ///   to determine the end of the slice.
    /// - For the last entry, uses `total_len` as the end of the slice.
    pub fn next_var(&self, idx: u32) -> Result<&'a [u8], CodecError> {
        let count = self.var_count()?;
        if idx >= count {
            return Err(CodecError::InvalidLength);
        }

        // Helper to read a u32 from VarEntry at the given entry index.
        let read_entry = |decoder: &Decoder<'a>, entry_idx: u32| -> Result<u32, CodecError> {
            let offset_in_entries = entry_idx.checked_mul(4).ok_or(CodecError::InvalidLength)?;
            let var_entry_abs = decoder
                .var_idx_offset
                .checked_add(offset_in_entries)
                .ok_or(CodecError::InvalidLength)?;
            let var_entry_end_abs = var_entry_abs
                .checked_add(4)
                .ok_or(CodecError::InvalidLength)?;

            // Entry must be within the VarEntry region and payload.
            if var_entry_end_abs > decoder.data_offset || var_entry_end_abs > decoder.total_len {
                return Err(CodecError::InvalidLength);
            }

            let start = usize::try_from(var_entry_abs).map_err(|_| CodecError::InvalidLength)?;
            let end = usize::try_from(var_entry_end_abs).map_err(|_| CodecError::InvalidLength)?;
            if end > decoder.buf.len() {
                return Err(CodecError::InvalidLength);
            }

            let bytes: [u8; 4] = decoder.buf[start..end]
                .try_into()
                .map_err(|_| CodecError::InvalidLength)?;
            Ok(u32::from_le_bytes(bytes))
        };

        let start_abs = read_entry(self, idx)?;
        let end_abs = if idx + 1 < count {
            read_entry(self, idx + 1)?
        } else {
            self.total_len
        };

        // Offsets must describe a non-empty (or zero-length) slice inside the Data region.
        if start_abs < self.data_offset || end_abs < start_abs || end_abs > self.total_len {
            return Err(CodecError::InvalidLength);
        }

        let start = usize::try_from(start_abs).map_err(|_| CodecError::InvalidLength)?;
        let end = usize::try_from(end_abs).map_err(|_| CodecError::InvalidLength)?;
        if end > self.buf.len() {
            return Err(CodecError::InvalidLength);
        }

        Ok(&self.buf[start..end])
    }
}
