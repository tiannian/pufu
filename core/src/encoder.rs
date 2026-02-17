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
        const HEADER_LEN: u32 = 12;

        let total_len = self.fixed.len() + self.var_length.len() * 4 + self.data.len();
        let var_entry_offset = self.fixed.len();
        let data_offset = var_entry_offset + self.var_length.len() * 4;

        let total_len_u32: u32 = total_len as u32 + HEADER_LEN;
        let var_entry_offset_u32: u32 = var_entry_offset as u32 + HEADER_LEN;
        let data_offset_u32: u32 = data_offset as u32 + HEADER_LEN;

        out.extend_from_slice(&total_len_u32.to_le_bytes());
        out.extend_from_slice(&var_entry_offset_u32.to_le_bytes());
        out.extend_from_slice(&data_offset_u32.to_le_bytes());
        out.extend_from_slice(&self.fixed);

        // Convert data-relative offsets (stored as lengths) to absolute payload offsets
        // VarEntry contains absolute offsets from byte 0 pointing into the Data region
        // Data region starts at absolute offset: HEADER_LEN + data_offset
        let mut current_offset = data_offset_u32;
        for &length in &self.var_length {
            // Write the absolute offset pointing to the start of this variable-length value
            out.extend_from_slice(&current_offset.to_le_bytes());
            // Advance by the length of this value for the next offset
            current_offset = current_offset.checked_add(length).expect("offset overflow");
        }
        out.extend_from_slice(&self.data);
    }
}
