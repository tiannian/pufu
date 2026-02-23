use crate::Config;

pub struct Encoder {
    pub(crate) config: Config,
    /// Bytes for the FixedRegion.
    pub(crate) fixed: Vec<u8>,
    /// Data-relative lengths; converted to payload-relative offsets on finalize.
    pub(crate) var_length: Vec<u32>,
    /// Variable-length data region.
    pub(crate) data: Vec<u8>,
}
