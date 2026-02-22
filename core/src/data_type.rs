//! Data type descriptors for pufu encoding.

use crate::Endian;

/// Describes how a type is encoded in the payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataMode {
    /// Fixed-width values that live in the fixed region.
    Fixed,
    /// Variable-length values with a single layer of offsets.
    Var1,
}

/// Defines how a type contributes fixed or variable data to an encoder.
pub trait DataType {
    /// Encoding mode for this type.
    const MODE: DataMode;
    /// Fixed byte length, used only for `Fixed` types.
    const LENGTH: usize = 0;

    /// Push fixed-width bytes into the fixed region.
    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
        let _ = (encoder_fixed, endian);
        panic!("push_fixed_data called for non-fixed data type");
    }

    /// Push variable-length bytes into the data region and record length.
    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        let _ = (var_length, data, endian);
        panic!("push_var1_data called for fixed data type");
    }
}

macro_rules! impl_fixed_data_type_for_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl DataType for $t {
                const MODE: DataMode = DataMode::Fixed;
                const LENGTH: usize = std::mem::size_of::<$t>();

                fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
                    match endian {
                        Endian::Little => encoder_fixed.extend_from_slice(&self.to_le_bytes()),
                        Endian::Big => encoder_fixed.extend_from_slice(&self.to_be_bytes()),
                        Endian::Native => encoder_fixed.extend_from_slice(&self.to_le_bytes()),
                    }
                }
            }

        )*
    };
}

impl_fixed_data_type_for_primitive!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl<T, const N: usize> DataType for [T; N]
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Fixed;
    const LENGTH: usize = T::LENGTH * N;

    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("fixed arrays require fixed data types");
        }
        for i in self {
            i.push_fixed_data(encoder_fixed, endian);
        }
    }
}

impl<T, const N: usize> DataType for &[T; N]
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Fixed;
    const LENGTH: usize = T::LENGTH * N;

    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("fixed arrays require fixed data types");
        }
        for i in self.iter() {
            i.push_fixed_data(encoder_fixed, endian);
        }
    }
}

impl<T, const N: usize> DataType for &mut [T; N]
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Fixed;
    const LENGTH: usize = T::LENGTH * N;

    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("fixed arrays require fixed data types");
        }
        for i in self.iter() {
            i.push_fixed_data(encoder_fixed, endian);
        }
    }
}

impl<T> DataType for Vec<T>
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Var1;

    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("var1 vectors require fixed element types");
        }
        let this: &[T] = self;
        this.push_var1_data(var_length, data, endian);
    }
}

impl<T> DataType for &Vec<T>
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Var1;

    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("var1 vectors require fixed element types");
        }
        let this: &[T] = self.as_slice();
        this.push_var1_data(var_length, data, endian);
    }
}

impl<T> DataType for &mut Vec<T>
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Var1;

    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("var1 vectors require fixed element types");
        }
        let this: &[T] = self.as_slice();
        this.push_var1_data(var_length, data, endian);
    }
}

impl<T> DataType for &[T]
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Var1;

    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("var1 slices require fixed element types");
        }
        let mut length = 0;

        for item in self.iter() {
            item.push_fixed_data(data, endian);
            length += T::LENGTH;
        }

        var_length.push(length as u32);
    }
}

impl<T> DataType for &mut [T]
where
    T: DataType,
{
    const MODE: DataMode = DataMode::Var1;

    fn push_var1_data(&self, var_length: &mut Vec<u32>, data: &mut Vec<u8>, endian: &Endian) {
        if T::MODE != DataMode::Fixed {
            panic!("var1 slices require fixed element types");
        }
        let this: &[T] = self;
        this.push_var1_data(var_length, data, endian);
    }
}
