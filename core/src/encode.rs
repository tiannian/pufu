use crate::{Encoder, FixedDataType, Var1DataType};

pub trait FieldEncode {
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder);
}

impl<T: FixedDataType> FieldEncode for T {
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.push_fixed_data(&mut e.fixed, &e.endian);
    }
}

impl<T: FixedDataType> FieldEncode for Vec<T> {
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        let this: &[T] = &self;
        this.encode_field::<IS_LAST_VAR>(e);
    }
}

impl<T: FixedDataType> FieldEncode for &[T] {
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        let mut length = 0;

        for item in self.iter() {
            item.push_fixed_data(&mut e.data, &e.endian);

            length += T::LENGTH;
        }

        e.var_length.push(length as u32);
    }
}

impl<T: FixedDataType> FieldEncode for &mut [T] {
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        let this: &[T] = &self;
        this.encode_field::<IS_LAST_VAR>(e);
    }
}
