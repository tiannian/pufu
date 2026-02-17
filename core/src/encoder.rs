//! Encoder for building binary payloads (no magic or version; see specs/0011-encoder.md).

use crate::zc::{Endian, FixedDataType, Var1DataType};

/// Encoder for building binary payloads. Accumulates fixed region, variable-entry index
/// (data-relative offsets), and data region. Does not write magic or version.
#[derive(Debug)]
pub struct Encoder {
    /// Bytes for the FixedRegion.
    pub fixed: Vec<u8>,
    /// Data-relative offsets; converted to payload-relative on finalize.
    pub var_length: Vec<u32>,
    /// Variable-length data region.
    pub data: Vec<u8>,
    /// Endianness of the fixed data.
    pub endian: Endian,
}

impl Encoder {
    /// Creates an empty Encoder.
    pub fn new(endian: Endian) -> Self {
        Self {
            fixed: vec![],
            var_length: vec![],
            data: vec![],
            endian,
        }
    }

    pub fn little() -> Self {
        Self::new(Endian::Little)
    }

    pub fn big() -> Self {
        Self::new(Endian::Big)
    }

    pub fn native() -> Self {
        Self::new(Endian::Native)
    }

    pub fn push_fixed_array<D>(&mut self, data: D)
    where
        D: FixedDataType,
    {
        data.push_fixed_data(&mut self.fixed, &self.endian);
    }

    pub fn push_var1_data<T, Var1>(&mut self, data: Var1)
    where
        T: FixedDataType,
        Var1: Var1DataType<T>,
    {
        data.push_var1_data(&mut self.var_length, &mut self.data, &self.endian);
    }

    pub fn push_var2_data<T, Var1, Var2>(&mut self, data: Var2)
    where
        T: FixedDataType,
        Var1: Var1DataType<T>,
        Var2: AsRef<[Var1]>,
    {
        let this: &[Var1] = data.as_ref();

        for item in this.iter() {
            item.push_var1_data(&mut self.var_length, &mut self.data, &self.endian);
        }
    }

    pub fn finalize(self, out: &mut Vec<u8>) {
        // Header fields (excluding magic and version): total_len (4) + var_entry_offset (4) = 8 bytes
        const HEADER_FIELDS_LEN: u32 = 8;

        // Calculate offsets relative to the first byte after magic+version (i.e., payload start)
        let fixed_len = self.fixed.len() as u32;
        let var_entry_len = self.var_length.len() as u32 * 4;
        let data_len = self.data.len() as u32;

        // total_len: from first byte of total_len field to end of payload
        // Includes: total_len (4) + var_entry_offset (4) + FixedRegion + VarEntry + Data
        let total_len = HEADER_FIELDS_LEN + fixed_len + var_entry_len + data_len;

        // var_entry_offset: offset from first byte after magic+version to VarEntry region
        // Includes: total_len (4) + var_entry_offset (4) + FixedRegion
        let var_entry_offset = HEADER_FIELDS_LEN + fixed_len;

        // Data region starts after VarEntry
        let data_start_offset = var_entry_offset + var_entry_len;

        // Write header fields (total_len and var_entry_offset)
        out.extend_from_slice(&total_len.to_le_bytes());
        out.extend_from_slice(&var_entry_offset.to_le_bytes());

        // Write FixedRegion
        out.extend_from_slice(&self.fixed);

        let mut current_data_offset = data_start_offset;
        for &length in &self.var_length {
            out.extend_from_slice(&current_data_offset.to_le_bytes());
            current_data_offset += length;
        }

        // Write Data region
        out.extend_from_slice(&self.data);
    }
}
