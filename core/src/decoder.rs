//! Decoder for reading binary payloads (see specs/0012-decoder.md).

use crate::{CodecError, Config, Endian};

/// Reads a u32 from the first 4 bytes of `bytes` using the given endianness.
fn read_u32_endian(bytes: &[u8], endian: Endian) -> Result<u32, CodecError> {
    let arr: [u8; 4] = bytes
        .get(0..4)
        .and_then(|s| s.try_into().ok())
        .ok_or(CodecError::InvalidLength)?;
    Ok(match endian {
        Endian::Little => u32::from_le_bytes(arr),
        Endian::Big => u32::from_be_bytes(arr),
        Endian::Native => u32::from_ne_bytes(arr),
    })
}

/// Decoder for reading binary payloads produced by `Encoder`.
///
/// Expects `buf` to start with the 8-byte header (total_len, var_entry_offset) as written by
/// `Encoder::finalize`; i.e. the first byte of `buf` is the first byte after magic+version.
#[derive(Debug, Clone)]
pub struct Decoder<'a> {
    /// Config used for magic, version, and endianness (endian not serialized).
    pub config: Config,
    /// Buffer containing the payload (after magic+version).
    pub buf: &'a [u8],
    pub total_len: u32,
    pub var_idx_offset: u32,
    pub data_offset: u32,
    pub fixed_cursor: u32,
    pub var_cursor: u32,
}

impl<'a> Decoder<'a> {
    const HEADER_LEN: u32 = 8;

    /// Creates a Decoder by parsing the header from `buf` using `config` for endianness.
    pub fn new(buf: &'a [u8], config: Config) -> Result<Self, CodecError> {
        if buf.len() < Self::HEADER_LEN as usize {
            return Err(CodecError::InvalidLength);
        }

        let endian = config.endian;
        let total_len = read_u32_endian(&buf[0..4], endian)?;
        let var_idx_offset = read_u32_endian(&buf[4..8], endian)?;

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
            read_u32_endian(&buf[start..end], endian)?
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
            config,
            buf,
            total_len,
            var_idx_offset,
            data_offset,
            fixed_cursor: 0,
            var_cursor: 0,
        })
    }

    /// Returns a reference to the Config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns the number of variable-length entries. This is `(data_offset - var_idx_offset) / 4`.
    pub fn var_count(&self) -> u32 {
        (self.data_offset - self.var_idx_offset) / 4
    }

    /// Reads the next `len` bytes from the FixedRegion, advancing `fixed_cursor`.
    pub fn next_fixed_bytes(&mut self, len: u32) -> Result<&'a [u8], CodecError> {
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
    pub fn next_var(&mut self) -> Result<&'a [u8], CodecError> {
        let idx = self.next_var_index()?;
        let count = self.var_count();

        let start_abs = self.read_entry(idx)?;
        let end_abs = if idx + 1 < count {
            self.read_entry(idx + 1)?
        } else {
            self.total_len
        };

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

    fn read_entry(&self, entry_idx: u32) -> Result<u32, CodecError> {
        let offset_in_entries = entry_idx.checked_mul(4).ok_or(CodecError::InvalidLength)?;
        let var_entry_abs = self
            .var_idx_offset
            .checked_add(offset_in_entries)
            .ok_or(CodecError::InvalidLength)?;
        let var_entry_end_abs = var_entry_abs
            .checked_add(4)
            .ok_or(CodecError::InvalidLength)?;

        if var_entry_end_abs > self.data_offset || var_entry_end_abs > self.total_len {
            return Err(CodecError::InvalidLength);
        }

        let start = usize::try_from(var_entry_abs).map_err(|_| CodecError::InvalidLength)?;
        let end = usize::try_from(var_entry_end_abs).map_err(|_| CodecError::InvalidLength)?;
        if end > self.buf.len() {
            return Err(CodecError::InvalidLength);
        }

        read_u32_endian(&self.buf[start..end], self.config.endian)
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
