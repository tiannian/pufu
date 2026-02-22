//! Encoding support for pufu payloads.

use crate::{DataMode, DataType, Encoder};

/// Encodes a single field into the provided encoder.
pub trait Encode {
    /// Encode this field, marking whether it is the last variable-length field.
    ///
    /// The const flag is used to enforce var2 layout constraints at compile time.
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder);
}

macro_rules! impl_field_encode_for_fixed_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl Encode for $t {
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

impl<T, const N: usize> Encode for [T; N]
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_fixed_data(&mut e.fixed, &e.endian);
    }
}

impl<T, const N: usize> Encode for &[T; N]
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_fixed_data(&mut e.fixed, &e.endian);
    }
}

impl<T, const N: usize> Encode for &mut [T; N]
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_fixed_data(&mut e.fixed, &e.endian);
    }
}

impl<T> Encode for Vec<T>
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        match T::MODE {
            DataMode::Fixed => {
                let mut length = 0;
                for item in self.iter() {
                    item.push_fixed_data(&mut e.data, &e.endian);
                    length += T::LENGTH;
                }
                e.var_length.push(length as u32);
            }
            DataMode::Var1 => {
                if !IS_LAST_VAR {
                    panic!("var2 vectors cannot be encoded as last variable field");
                }
                for item in self.iter() {
                    item.push_var1_data(&mut e.var_length, &mut e.data, &e.endian);
                }
            }
        }
    }
}

impl<T> Encode for &Vec<T>
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        <Vec<T> as Encode>::encode_field::<IS_LAST_VAR>(self, e);
    }
}

impl<T> Encode for &mut Vec<T>
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        <Vec<T> as Encode>::encode_field::<IS_LAST_VAR>(self, e);
    }
}

impl<T> Encode for &[T]
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        match T::MODE {
            DataMode::Fixed => {
                let mut length = 0;
                for item in self.iter() {
                    item.push_fixed_data(&mut e.data, &e.endian);
                    length += T::LENGTH;
                }
                e.var_length.push(length as u32);
            }
            DataMode::Var1 => {
                if !IS_LAST_VAR {
                    panic!("var2 vectors cannot be encoded as last variable field");
                }
                for item in self.iter() {
                    item.push_var1_data(&mut e.var_length, &mut e.data, &e.endian);
                }
            }
        }
    }
}

impl<T> Encode for &mut [T]
where
    T: DataType,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        let this: &[T] = self;
        <&[T] as Encode>::encode_field::<IS_LAST_VAR>(&this, e);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Encode, Encoder};

    #[test]
    #[should_panic(expected = "var2 vectors cannot be encoded as last variable field")]
    fn rejects_var2_when_not_marked_last_var() {
        let mut encoder = Encoder::little();
        let value: Vec<Vec<u16>> = vec![vec![1, 2], vec![3]];
        value.encode_field::<false>(&mut encoder);
    }
}
