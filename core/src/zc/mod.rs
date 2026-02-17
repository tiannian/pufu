mod fixed;
pub use fixed::FixedDataType;

mod var1;
pub use var1::Var1DataType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
    Native,
}
