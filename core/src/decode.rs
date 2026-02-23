use crate::{Decoder, Result};

pub trait Decode {
    type View<'a>
    where
        Self: 'a;

    // Required method
    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>>;
}
