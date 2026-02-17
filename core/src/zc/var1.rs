use crate::zc::{Endian, FixedDataType};

pub trait Var1DataType<T: FixedDataType> {
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian);
}

impl<T: FixedDataType> Var1DataType<T> for Vec<T> {
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        let this: &[T] = &self;
        this.push_var1_data(var_length, data, endian);
    }
}

impl<T: FixedDataType> Var1DataType<T> for &[T] {
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        let mut length = 0;

        for item in self.iter() {
            item.push_fixed_data(data, endian);

            length += T::LENGTH;
        }

        var_length.push(length as u32);
    }
}

impl<T: FixedDataType> Var1DataType<T> for &mut [T] {
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        let this = self.as_ref();
        this.push_var1_data(var_length, data, endian);
    }
}
