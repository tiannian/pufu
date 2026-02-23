use crate::{ElementRank1, Encoder, NotElementRank0};

pub trait Encode {
    // Required method
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder);
}

impl<T: ElementRank1> Encode for T {
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        self.encode(e);
    }
}

impl<T> Encode for Vec<T>
where
    T: ElementRank1 + NotElementRank0,
{
    fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder) {
        for item in self {
            item.encode_field(e);
        }
    }
}
