//! Fixed-width decoding helpers for pufu payloads.

use crate::{CodecError, Endian};

/// Decodes fixed-width values from a byte slice.
pub(crate) trait FixedDecode: Sized {
    /// Fixed byte length for this type.
    const LENGTH: usize;
    /// Decode from a fixed-length byte slice with the given endianness.
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

impl<T, const N: usize> FixedDecode for [T; N]
where
    T: FixedDecode,
{
    const LENGTH: usize = T::LENGTH * N;

    fn decode(bytes: &[u8], endian: Endian) -> Result<Self, CodecError> {
        if T::LENGTH == 0 || !bytes.len().is_multiple_of(T::LENGTH) {
            return Err(CodecError::InvalidLength);
        }

        let mut items = Vec::with_capacity(N);
        for chunk in bytes.chunks_exact(T::LENGTH) {
            items.push(T::decode(chunk, endian)?);
        }

        items.try_into().map_err(|_| CodecError::InvalidLength)
    }
}
