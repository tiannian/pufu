use crate::{CodecError, Endian};

pub(crate) trait FixedDecode: Sized {
    const LENGTH: usize;
    fn decode(bytes: &[u8], endian: Endian) -> Result<Self, CodecError>;
}

macro_rules! impl_fixed_decode_for_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl FixedDecode for $t {
                const LENGTH: usize = std::mem::size_of::<$t>();

                fn decode(bytes: &[u8], endian: Endian) -> Result<Self, CodecError> {
                    let array: [u8; std::mem::size_of::<$t>()] = bytes
                        .try_into()
                        .map_err(|_| CodecError::InvalidLength)?;
                    Ok(match endian {
                        Endian::Little => <$t>::from_le_bytes(array),
                        Endian::Big => <$t>::from_be_bytes(array),
                        Endian::Native => <$t>::from_le_bytes(array),
                    })
                }
            }
        )*
    };
}

impl_fixed_decode_for_primitive!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
