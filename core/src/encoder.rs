//! Encoder for building binary payloads (see specs/0011-encoder.md).

use crate::{CodecError, Config, Endian};

/// Writes `value` as 4 bytes into `out` using the given endianness (not serialized on wire).
fn write_u32_endian(out: &mut Vec<u8>, value: u32, endian: Endian) {
    let bytes = match endian {
        Endian::Little => value.to_le_bytes(),
        Endian::Big => value.to_be_bytes(),
        Endian::Native => value.to_ne_bytes(),
    };
    out.extend_from_slice(&bytes);
}

/// Encoder for building binary payloads. Holds Config (magic, version, endian); accumulates
/// fixed region, variable-entry lengths, and data region.
#[derive(Debug)]
pub struct Encoder {
    /// Config for magic, version, and endianness (endian not serialized).
    pub config: Config,
    /// Bytes for the FixedRegion.
    pub fixed: Vec<u8>,
    /// Data-relative lengths; converted to payload-relative offsets on finalize.
    pub var_length: Vec<u32>,
    /// Variable-length data region.
    pub data: Vec<u8>,
}

impl Encoder {
    /// Creates an Encoder with the given Config and empty regions.
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            fixed: vec![],
            var_length: vec![],
            data: vec![],
        }
    }

    /// Returns a reference to the Config (e.g. for nested encoders).
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Finalizes the payload into `out` (no magic or version). Uses config endian for u32 fields.
    pub fn finalize(self, out: &mut Vec<u8>) -> Result<(), CodecError> {
        const HEADER_FIELDS_LEN: u32 = 8;

        let fixed_len = u32::try_from(self.fixed.len()).map_err(|_| CodecError::InvalidLength)?;
        let var_entry_len = self
            .var_length
            .len()
            .checked_mul(4)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(CodecError::InvalidLength)?;
        let data_len = u32::try_from(self.data.len()).map_err(|_| CodecError::InvalidLength)?;

        let total_len = HEADER_FIELDS_LEN
            .checked_add(fixed_len)
            .and_then(|n| n.checked_add(var_entry_len))
            .and_then(|n| n.checked_add(data_len))
            .ok_or(CodecError::InvalidLength)?;
        let var_entry_offset = HEADER_FIELDS_LEN
            .checked_add(fixed_len)
            .ok_or(CodecError::InvalidLength)?;
        let data_start_offset = var_entry_offset
            .checked_add(var_entry_len)
            .ok_or(CodecError::InvalidLength)?;

        let endian = self.config.endian;
        write_u32_endian(out, total_len, endian);
        write_u32_endian(out, var_entry_offset, endian);
        out.extend_from_slice(&self.fixed);

        let mut current_data_offset = data_start_offset;
        for &length in &self.var_length {
            write_u32_endian(out, current_data_offset, endian);
            current_data_offset = current_data_offset
                .checked_add(length)
                .ok_or(CodecError::InvalidLength)?;
        }
        out.extend_from_slice(&self.data);
        Ok(())
    }

    /// Writes full payload: 4-byte magic, 1-byte version from config, then layout as in `finalize`.
    pub fn finalize_with_magic_version(self, out: &mut Vec<u8>) -> Result<(), CodecError> {
        out.extend_from_slice(&self.config.magic);
        out.push(self.config.version);
        self.finalize(out)
    }
}

#[cfg(test)]
mod tests {
    use super::Encoder;
    use crate::{Config, Encode};

    #[test]
    fn encode_fixed_and_var1_vec_fixed() {
        let mut encoder = Encoder::new(Config::default());

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
        encoder.finalize(&mut out).expect("finalize");
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
        let mut encoder = Encoder::new(Config::default());
        let mut outer: Vec<Vec<u16>> = vec![vec![1, 2], vec![3]];

        (&outer).encode_field::<true>(&mut encoder);
        assert_eq!(encoder.var_length, vec![4, 2]);
        assert_eq!(encoder.data, vec![0x01, 0x00, 0x02, 0x00, 0x03, 0x00]);

        let mut out = Vec::new();
        encoder.finalize(&mut out).expect("finalize");
        assert_eq!(
            out,
            vec![
                0x16, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x14, 0x00,
                0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00,
            ]
        );

        let mut encoder = Encoder::new(Config::default());
        (&mut outer).encode_field::<true>(&mut encoder);
        assert_eq!(encoder.var_length, vec![4, 2]);
        assert_eq!(encoder.data, vec![0x01, 0x00, 0x02, 0x00, 0x03, 0x00]);
    }

    #[test]
    #[should_panic(expected = "var1 vectors require fixed element types")]
    fn rejects_var3_vec_vec_vec_u8() {
        let mut encoder = Encoder::new(Config::default());
        let value: Vec<Vec<Vec<u8>>> = vec![vec![vec![1]]];
        value.encode_field::<true>(&mut encoder);
    }
}
