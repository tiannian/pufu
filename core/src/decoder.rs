use crate::{Config, Result};

pub struct Decoder<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) config: Config,
    pub(crate) fixed_cursor: u32,
    pub(crate) var_cursor: u32,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8], config: Config) -> Self {
        Self {
            buf,
            config,
            fixed_cursor: 0,
            var_cursor: 0,
        }
    }

    pub fn next_fixed_bytes(&mut self, len: u32) -> Result<&'a [u8]> {
        Ok(&self.buf[..len as usize])
    }
}
