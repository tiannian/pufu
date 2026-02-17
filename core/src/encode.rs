use crate::{Encoder, FixedDataType, Var1DataType};

pub trait FieldEncode {
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder);
}

macro_rules! impl_field_encode_for_fixed_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl FieldEncode for $t {
                fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
                    self.push_fixed_data(&mut e.fixed, &e.endian);
                }
            }
        )*
    };
}

impl_field_encode_for_fixed_primitive!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl<T, const N: usize> FieldEncode for [T; N]
where
    T: FixedDataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_fixed_data(&mut e.fixed, &e.endian);
    }
}

impl<T> FieldEncode for Vec<T>
where
    T: FixedDataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_var1_data(&mut e.var_length, &mut e.data, &e.endian);
    }
}

impl<T> FieldEncode for &[T]
where
    T: FixedDataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_var1_data(&mut e.var_length, &mut e.data, &e.endian);
    }
}

impl<T> FieldEncode for &mut [T]
where
    T: FixedDataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_var1_data(&mut e.var_length, &mut e.data, &e.endian);
    }
}
