use crate::{CodecError, Decoder, Endian};

pub trait FieldDecode {
    type View<'a>
    where
        Self: 'a;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError>;
}

trait FixedDecode: Sized {
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

fn decode_fixed_value<'a, T>(decoder: &mut Decoder<'a>) -> Result<T, CodecError>
where
    T: FixedDecode,
{
    let bytes = decoder.next_fixed_bytes(T::LENGTH as u32)?;
    T::decode(bytes, decoder.endian)
}

fn decode_fixed_slice<T>(bytes: &[u8], endian: Endian) -> Result<Vec<T>, CodecError>
where
    T: FixedDecode,
{
    if T::LENGTH == 0 || !bytes.len().is_multiple_of(T::LENGTH) {
        return Err(CodecError::InvalidLength);
    }

    let mut out = Vec::with_capacity(bytes.len() / T::LENGTH);
    for chunk in bytes.chunks_exact(T::LENGTH) {
        out.push(T::decode(chunk, endian)?);
    }
    Ok(out)
}

macro_rules! impl_field_decode_for_fixed_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl FieldDecode for $t {
                type View<'a> = $t;

                fn decode_field<'a, const IS_LAST_VAR: bool>(
                    decoder: &mut Decoder<'a>,
                ) -> Result<Self::View<'a>, CodecError> {
                    let _ = IS_LAST_VAR;
                    decode_fixed_value::<$t>(decoder)
                }
            }
        )*
    };
}

impl_field_decode_for_fixed_primitive!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl<T, const N: usize> FieldDecode for [T; N]
where
    T: FixedDecode + 'static,
{
    type View<'a>
        = [T; N]
    where
        T: 'a;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError> {
        let _ = IS_LAST_VAR;
        let len = T::LENGTH.checked_mul(N).ok_or(CodecError::InvalidLength)? as u32;
        let bytes = decoder.next_fixed_bytes(len)?;
        let items = decode_fixed_slice::<T>(bytes, decoder.endian)?;
        items.try_into().map_err(|_| CodecError::InvalidLength)
    }
}

impl<T> FieldDecode for Vec<T>
where
    T: FixedDecode + 'static,
{
    type View<'a>
        = Vec<T>
    where
        T: 'a;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError> {
        let _ = IS_LAST_VAR;
        let bytes = decoder.next_var()?;
        decode_fixed_slice::<T>(bytes, decoder.endian)
    }
}

impl<T> FieldDecode for Vec<Vec<T>>
where
    T: FixedDecode + 'static,
{
    type View<'a>
        = Vec<Vec<T>>
    where
        T: 'a;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError> {
        if !IS_LAST_VAR {
            return Err(CodecError::InvalidLength);
        }

        let mut out = Vec::new();
        let count = decoder.var_count();
        while decoder.var_cursor < count {
            let bytes = decoder.next_var()?;
            out.push(decode_fixed_slice::<T>(bytes, decoder.endian)?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::FieldDecode;
    use crate::{Decoder, Encoder, FieldEncode};

    #[test]
    fn decode_fixed_and_var1_vec_fixed() {
        let mut encoder = Encoder::little();

        let fixed_u8: u8 = 0xaa;
        let fixed_array: [u16; 2] = [0x0102, 0x0304];
        let var_vec: Vec<u32> = vec![0x0a0b0c0d, 0x01020304];

        fixed_u8.encode_field::<false>(&mut encoder);
        fixed_array.encode_field::<false>(&mut encoder);
        var_vec.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        let mut decoder = Decoder::new(&out).expect("decoder");

        let decoded_u8 = u8::decode_field::<false>(&mut decoder).expect("u8");
        let decoded_array = <[u16; 2]>::decode_field::<false>(&mut decoder).expect("array");
        let decoded_vec = Vec::<u32>::decode_field::<true>(&mut decoder).expect("vec");

        assert_eq!(decoded_u8, fixed_u8);
        assert_eq!(decoded_array, fixed_array);
        assert_eq!(decoded_vec, var_vec);
    }

    #[test]
    fn decode_var2_vec_vec_fixed() {
        let mut encoder = Encoder::little();
        let outer: Vec<Vec<u16>> = vec![vec![1, 2], vec![3]];

        outer.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        let mut decoder = Decoder::new(&out).expect("decoder");

        let decoded = Vec::<Vec<u16>>::decode_field::<true>(&mut decoder).expect("vec vec");
        assert_eq!(decoded, outer);
    }
}
