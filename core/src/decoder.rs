//! Decoder for reading binary payloads (no magic or version; see specs/0012-decoder.md).

use crate::{CodecError, Endian};

/// Decoder for reading binary payloads produced by `Encoder`.
///
/// The layout is:
/// - `total_len` (u32, LE)
/// - `var_entry_offset` (u32, LE)
/// - FixedRegion bytes
/// - VarEntry region (u32 LE offsets, payload-relative)
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
    /// Current cursor in the FixedRegion, as an offset relative to the FixedRegion start.
    pub fixed_cursor: u32,
    /// Current cursor in the VarEntry region (index into var entries).
    pub var_cursor: u32,
    /// Endianness for decoding fixed data.
    pub endian: Endian,
}

impl<'a> Decoder<'a> {
    const HEADER_LEN: u32 = 8;

    /// Creates a new Decoder by parsing the header from `buf`.
    ///
    /// Expects the first 8 bytes of `buf` to be:
    /// - `total_len` (u32 LE)
    /// - `var_entry_offset` (u32 LE)
    ///
    /// Validates that these values describe a layout fully contained within `buf`.
    pub fn new(buf: &'a [u8]) -> Result<Self, CodecError> {
        Self::with_endian(buf, Endian::Little)
    }

    /// Creates a decoder with an explicit fixed-data endianness.
    pub fn with_endian(buf: &'a [u8], endian: Endian) -> Result<Self, CodecError> {
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
        let total_len_usize = total_len as usize;
        if total_len_usize > buf.len() {
            return Err(CodecError::InvalidLength);
        }
        if var_idx_offset < Self::HEADER_LEN {
            return Err(CodecError::InvalidLength);
        }
        if var_idx_offset > total_len {
            return Err(CodecError::InvalidLength);
        }

        let data_offset = if total_len == var_idx_offset {
            var_idx_offset
        } else {
            let start = usize::try_from(var_idx_offset).map_err(|_| CodecError::InvalidLength)?;
            let end = start.checked_add(4).ok_or(CodecError::InvalidLength)?;
            if end > buf.len() {
                return Err(CodecError::InvalidLength);
            }
            let bytes: [u8; 4] = buf[start..end]
                .try_into()
                .map_err(|_| CodecError::InvalidLength)?;
            u32::from_le_bytes(bytes)
        };

        if data_offset < var_idx_offset {
            return Err(CodecError::InvalidLength);
        }
        if data_offset > total_len {
            return Err(CodecError::InvalidLength);
        }
        if total_len > var_idx_offset && data_offset == var_idx_offset {
            return Err(CodecError::InvalidLength);
        }
        if (data_offset - var_idx_offset) % 4 != 0 {
            return Err(CodecError::InvalidLength);
        }

        Ok(Self {
            buf,
            total_len,
            var_idx_offset,
            data_offset,
            fixed_cursor: 0,
            var_cursor: 0,
            endian,
        })
    }

    /// Creates a decoder that interprets fixed values as little-endian.
    pub fn little(buf: &'a [u8]) -> Result<Self, CodecError> {
        Self::with_endian(buf, Endian::Little)
    }

    /// Creates a decoder that interprets fixed values as big-endian.
    pub fn big(buf: &'a [u8]) -> Result<Self, CodecError> {
        Self::with_endian(buf, Endian::Big)
    }

    /// Creates a decoder that interprets fixed values as native-endian.
    pub fn native(buf: &'a [u8]) -> Result<Self, CodecError> {
        Self::with_endian(buf, Endian::Native)
    }

    /// Returns the number of variable-length entries in the VarEntry region.
    ///
    /// This is `(data_offset - var_idx_offset) / 4`.
    pub fn var_count(&self) -> u32 {
        (self.data_offset - self.var_idx_offset) / 4
    }

    /// Reads the next `len` bytes from the FixedRegion, advancing `fixed_cursor`.
    pub fn next_fixed_bytes(&mut self, len: u32) -> Result<&'a [u8], CodecError> {
        // Remaining bytes in the FixedRegion from the current cursor.
        let fixed_len = self
            .var_idx_offset
            .checked_sub(Self::HEADER_LEN)
            .ok_or(CodecError::InvalidLength)?;
        let remaining = fixed_len
            .checked_sub(self.fixed_cursor)
            .ok_or(CodecError::InvalidLength)?;
        if len > remaining {
            return Err(CodecError::InvalidLength);
        }

        let start_abs = self
            .fixed_cursor
            .checked_add(Self::HEADER_LEN)
            .ok_or(CodecError::InvalidLength)?;
        let end_abs = start_abs
            .checked_add(len)
            .ok_or(CodecError::InvalidLength)?;

        let start = usize::try_from(start_abs).map_err(|_| CodecError::InvalidLength)?;
        let end = usize::try_from(end_abs).map_err(|_| CodecError::InvalidLength)?;

        self.fixed_cursor = self
            .fixed_cursor
            .checked_add(len)
            .ok_or(CodecError::InvalidLength)?;
        Ok(&self.buf[start..end])
    }

    /// Reads the next variable-length value using VarEntry offsets.
    ///
    /// - Reads the current VarEntry `u32` as an absolute payload offset into the Data region.
    /// - For all but the last entry, also reads the next VarEntry `u32` offset to determine
    ///   the end of the slice.
    /// - For the last entry, uses `total_len` as the end of the slice.
    pub fn next_var(&mut self) -> Result<&'a [u8], CodecError> {
        let idx = self.next_var_index()?;
        let count = self.var_count();

        let start_abs = self.read_entry(idx)?;
        let end_abs = if idx + 1 < count {
            self.read_entry(idx + 1)?
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

    /// Reads a `u32` VarEntry at the given entry index.
    fn read_entry(&self, entry_idx: u32) -> Result<u32, CodecError> {
        let offset_in_entries = entry_idx.checked_mul(4).ok_or(CodecError::InvalidLength)?;
        let var_entry_abs = self
            .var_idx_offset
            .checked_add(offset_in_entries)
            .ok_or(CodecError::InvalidLength)?;
        let var_entry_end_abs = var_entry_abs
            .checked_add(4)
            .ok_or(CodecError::InvalidLength)?;

        // Entry must be within the VarEntry region and payload.
        if var_entry_end_abs > self.data_offset || var_entry_end_abs > self.total_len {
            return Err(CodecError::InvalidLength);
        }

        let start = usize::try_from(var_entry_abs).map_err(|_| CodecError::InvalidLength)?;
        let end = usize::try_from(var_entry_end_abs).map_err(|_| CodecError::InvalidLength)?;
        if end > self.buf.len() {
            return Err(CodecError::InvalidLength);
        }

        let bytes: [u8; 4] = self.buf[start..end]
            .try_into()
            .map_err(|_| CodecError::InvalidLength)?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Returns the next VarEntry index and advances the cursor.
    pub fn next_var_index(&mut self) -> Result<u32, CodecError> {
        let count = self.var_count();
        if self.var_cursor >= count {
            return Err(CodecError::InvalidLength);
        }
        let idx = self.var_cursor;
        self.var_cursor += 1;
        Ok(idx)
    }
}
