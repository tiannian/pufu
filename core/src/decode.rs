use crate::fixed_decode::FixedDecode;
use crate::{CodecError, Decoder, Endian};

pub trait Decode {
    type View<'a>
    where
        Self: 'a;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError>;
}

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

fn decode_fixed_slice_u8_ref(bytes: &[u8]) -> Result<&[u8], CodecError> {
    let _ = bytes;
    Ok(bytes)
}

trait NotU8 {}

macro_rules! impl_not_u8_for_primitive {
    ($($t:ty),* $(,)?) => {
        $(impl NotU8 for $t {})*
    };
}

impl_not_u8_for_primitive!(u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl<T, const N: usize> NotU8 for [T; N] where T: FixedDecode {}

macro_rules! impl_field_decode_for_fixed_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl Decode for $t {
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

impl<T, const N: usize> Decode for [T; N]
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

impl<T> Decode for Vec<T>
where
    T: FixedDecode + NotU8 + 'static,
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

impl<T> Decode for Vec<Vec<T>>
where
    T: FixedDecode + NotU8 + 'static,
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

impl Decode for Vec<u8> {
    type View<'a>
        = &'a [u8]
    where
        u8: 'a;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError> {
        let _ = IS_LAST_VAR;
        let bytes = decoder.next_var()?;
        decode_fixed_slice_u8_ref(bytes)
    }
}

impl Decode for Vec<Vec<u8>> {
    type View<'a>
        = Vec<&'a [u8]>
    where
        u8: 'a;

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
            out.push(decode_fixed_slice_u8_ref(bytes)?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::Decode;
    use crate::{CodecError, Decoder, Encode, Encoder, Endian};

    #[test]
    fn decode_fixed_and_var1_vec_fixed() {
        let mut encoder = Encoder::little();

        let fixed_u8: u8 = 0xaa;
        let fixed_array: [u16; 2] = [0x0102, 0x0304];
        let var_vec: Vec<u8> = vec![0x0a, 0x0b, 0x0c, 0x0d];

        fixed_u8.encode_field::<false>(&mut encoder);
        fixed_array.encode_field::<false>(&mut encoder);
        var_vec.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        let mut decoder = Decoder::new(&out).expect("decoder");

        let decoded_u8 = u8::decode_field::<false>(&mut decoder).expect("u8");
        let decoded_array = <[u16; 2]>::decode_field::<false>(&mut decoder).expect("array");
        let decoded_vec = Vec::<u8>::decode_field::<true>(&mut decoder).expect("vec");

        assert_eq!(decoded_u8, fixed_u8);
        assert_eq!(decoded_array, fixed_array);
        assert_eq!(decoded_vec, var_vec.as_slice());
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

    #[test]
    fn decode_fixed_array_rejects_short_fixed_region() {
        let buf = vec![8, 0, 0, 0, 8, 0, 0, 0];
        let mut decoder = Decoder::new(&buf).expect("decoder");
        let decoded = <[u16; 2]>::decode_field::<false>(&mut decoder);
        assert_eq!(decoded, Err(CodecError::InvalidLength));
    }

    #[test]
    fn decode_vec_fixed_rejects_non_multiple_length() {
        let mut buf = Vec::new();
        let total_len: u32 = 15;
        let var_idx_offset: u32 = 8;
        let data_offset: u32 = 12;
        buf.extend_from_slice(&total_len.to_le_bytes());
        buf.extend_from_slice(&var_idx_offset.to_le_bytes());
        buf.extend_from_slice(&data_offset.to_le_bytes());
        buf.extend_from_slice(&[0x01, 0x02, 0x03]);

        let mut decoder = Decoder::new(&buf).expect("decoder");
        let decoded = Vec::<u16>::decode_field::<true>(&mut decoder);
        assert_eq!(decoded, Err(CodecError::InvalidLength));
    }

    #[test]
    fn decode_vec_vec_requires_last_var() {
        let buf = vec![8, 0, 0, 0, 8, 0, 0, 0];
        let mut decoder = Decoder::new(&buf).expect("decoder");
        let decoded = Vec::<Vec<u16>>::decode_field::<false>(&mut decoder);
        assert_eq!(decoded, Err(CodecError::InvalidLength));
    }

    #[test]
    fn decode_fixed_big_endian() {
        let mut encoder = Encoder::big();
        let fixed_u16: u16 = 0x0102;
        fixed_u16.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);

        let mut decoder = Decoder::with_endian(&out, Endian::Big).expect("decoder");
        let decoded_u16 = u16::decode_field::<true>(&mut decoder).expect("u16");
        assert_eq!(decoded_u16, fixed_u16);
    }

    #[test]
    fn decode_only_fixed_fields() {
        let mut encoder = Encoder::little();
        let a: u8 = 0xaa;
        let b: u32 = 0x01020304;
        let c: [u16; 2] = [0x0a0b, 0x0c0d];

        a.encode_field::<false>(&mut encoder);
        b.encode_field::<false>(&mut encoder);
        c.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);

        let mut decoder = Decoder::new(&out).expect("decoder");
        let decoded_a = u8::decode_field::<false>(&mut decoder).expect("a");
        let decoded_b = u32::decode_field::<false>(&mut decoder).expect("b");
        let decoded_c = <[u16; 2]>::decode_field::<true>(&mut decoder).expect("c");

        assert_eq!(decoded_a, a);
        assert_eq!(decoded_b, b);
        assert_eq!(decoded_c, c);
    }
}
