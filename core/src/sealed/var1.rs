use crate::{Endian, FixedDataType};

pub mod var1_sealed {
    pub trait Var1DataType {}
}

pub trait Var1DataType: var1_sealed::Var1DataType {
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian);
}

impl<T: FixedDataType> var1_sealed::Var1DataType for Vec<T> {}

impl<T> Var1DataType for Vec<T>
where
    T: FixedDataType,
{
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        let this: &[T] = self;
        this.push_var1_data(var_length, data, endian);
    }
}

impl<T> var1_sealed::Var1DataType for &[T] where T: FixedDataType {}

impl<T> Var1DataType for &[T]
where
    T: FixedDataType,
{
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        let mut length = 0;

        for item in self.iter() {
            item.push_fixed_data(data, endian);

            length += T::LENGTH;
        }

        var_length.push(length as u32);
    }
}

impl<T> var1_sealed::Var1DataType for &mut [T] where T: FixedDataType {}

impl<T> Var1DataType for &mut [T]
where
    T: FixedDataType,
{
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        let this: &[T] = self;
        this.push_var1_data(var_length, data, endian);
    }
}
