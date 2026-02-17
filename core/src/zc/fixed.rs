use crate::zc::Endian;

pub trait FixedDataType {
    const LENGTH: usize;

    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian);
}

macro_rules! impl_fixed_data_type_for_primitive {
    ($($t:ty),*) => {
        $(
            impl FixedDataType for $t {
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

impl<T: FixedDataType, const N: usize> FixedDataType for [T; N] {
    const LENGTH: usize = T::LENGTH * N;

    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
        for i in self {
            i.push_fixed_data(encoder_fixed, endian);
        }
    }
}

impl<T: FixedDataType, const N: usize> FixedDataType for &[T; N] {
    const LENGTH: usize = T::LENGTH * N;

    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
        for i in self.iter() {
            i.push_fixed_data(encoder_fixed, endian);
        }
    }
}

impl<T: FixedDataType, const N: usize> FixedDataType for &mut [T; N] {
    const LENGTH: usize = T::LENGTH * N;

    fn push_fixed_data(&self, encoder_fixed: &mut Vec<u8>, endian: &Endian) {
        for i in self.iter() {
            i.push_fixed_data(encoder_fixed, endian);
        }
    }
}
