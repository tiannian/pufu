//! Encoder for building binary payloads (no magic or version; see specs/0011-encoder.md).

use crate::Endian;

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

#[cfg(test)]
mod tests {
    use super::Encoder;
    use crate::FieldEncode;

    #[test]
    fn encode_fixed_and_var1_vec_fixed() {
        let mut encoder = Encoder::little();

        let fixed_u8: u8 = 0xaa;
        let fixed_array: [u16; 2] = [0x0102, 0x0304];
        let var_vec: Vec<u32> = vec![0x0a0b0c0d, 0x01020304];

        fixed_u8.encode_field::<false>(&mut encoder);
        fixed_array.encode_field::<false>(&mut encoder);
        var_vec.encode_field::<true>(&mut encoder);

        assert_eq!(encoder.fixed, vec![0xaa, 0x02, 0x01, 0x04, 0x03]);
        assert_eq!(encoder.var_length, vec![8]);
        assert_eq!(
            encoder.data,
            vec![0x0d, 0x0c, 0x0b, 0x0a, 0x04, 0x03, 0x02, 0x01]
        );

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        assert_eq!(
            out,
            vec![
                0x19, 0x00, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00, 0xaa, 0x02, 0x01, 0x04, 0x03, 0x11,
                0x00, 0x00, 0x00, 0x0d, 0x0c, 0x0b, 0x0a, 0x04, 0x03, 0x02, 0x01,
            ]
        );
    }

    #[test]
    fn encode_var2_vec_vec_fixed() {
        let mut encoder = Encoder::little();
        let mut outer: Vec<Vec<u16>> = vec![vec![1, 2], vec![3]];

        (&outer).encode_field::<false>(&mut encoder);
        assert_eq!(encoder.var_length, vec![4, 2]);
        assert_eq!(encoder.data, vec![0x01, 0x00, 0x02, 0x00, 0x03, 0x00]);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        assert_eq!(
            out,
            vec![
                0x16, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x14, 0x00,
                0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00,
            ]
        );

        let mut encoder = Encoder::little();
        (&mut outer).encode_field::<false>(&mut encoder);
        assert_eq!(encoder.var_length, vec![4, 2]);
        assert_eq!(encoder.data, vec![0x01, 0x00, 0x02, 0x00, 0x03, 0x00]);
    }
}
