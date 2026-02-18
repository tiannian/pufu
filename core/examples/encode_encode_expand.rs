use pufu_core::{CodecError, Decode, Decoder, Encode, Encoder};

#[derive(Debug, PartialEq, Eq)]
struct EncodeEncodeExpand {
    fixed_a: u32,
    fixed_b: u16,
    var1_a: Vec<u16>,
    var1_c: Vec<u8>,
    fixed_c: u8,
    var1_b: Vec<u16>,
    fixed_d: u64,
    var2: Vec<Vec<u8>>,
}

#[derive(Debug, PartialEq, Eq)]
struct EncodeEncodeExpandView<'a> {
    fixed_a: u32,
    fixed_b: u16,
    var1_a: Vec<u16>,
    var1_c: &'a [u8],
    fixed_c: u8,
    var1_b: Vec<u16>,
    fixed_d: u64,
    var2: Vec<&'a [u8]>,
}

impl EncodeEncodeExpand {
    fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::little();

        self.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        out
    }

    fn decode(buf: &[u8]) -> Result<EncodeEncodeExpandView<'_>, CodecError> {
        let mut decoder = Decoder::new(buf)?;

        Self::decode_field::<true>(&mut decoder)
    }
}

impl Encode for EncodeEncodeExpand {
    fn encode_field<const IS_LAST_VAR: bool>(&self, encoder: &mut Encoder) {
        let _ = IS_LAST_VAR;
        self.fixed_a.encode_field::<false>(encoder);
        self.fixed_b.encode_field::<false>(encoder);
        self.var1_a.encode_field::<false>(encoder);
        self.var1_c.encode_field::<false>(encoder);
        self.fixed_c.encode_field::<false>(encoder);
        self.var1_b.encode_field::<false>(encoder);
        self.fixed_d.encode_field::<false>(encoder);
        self.var2.encode_field::<true>(encoder);
    }
}

impl Decode for EncodeEncodeExpand {
    type View<'a> = EncodeEncodeExpandView<'a>;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError> {
        let _ = IS_LAST_VAR;
        let fixed_a = u32::decode_field::<false>(decoder)?;
        let fixed_b = u16::decode_field::<false>(decoder)?;
        let var1_a = Vec::<u16>::decode_field::<false>(decoder)?;
        let var1_c = Vec::<u8>::decode_field::<false>(decoder)?;
        let fixed_c = u8::decode_field::<false>(decoder)?;
        let var1_b = Vec::<u16>::decode_field::<false>(decoder)?;
        let fixed_d = u64::decode_field::<false>(decoder)?;
        let var2 = Vec::<Vec<u8>>::decode_field::<true>(decoder)?;

        Ok(EncodeEncodeExpandView {
            fixed_a,
            fixed_b,
            var1_a,
            var1_c,
            fixed_c,
            var1_b,
            fixed_d,
            var2,
        })
    }
}

fn main() -> Result<(), CodecError> {
    let value = EncodeEncodeExpand {
        fixed_a: 0x0102_0304,
        fixed_b: 0x0506,
        var1_a: vec![10, 20, 30],
        var1_c: vec![9, 8, 7, 6],
        fixed_c: 0x07,
        var1_b: vec![0x0a0b, 0x0c0d],
        fixed_d: 0x1213_1415_1617_1819,
        var2: vec![vec![1, 2, 3], vec![4, 5]],
    };

    let encoded = value.encode();
    let decoded = EncodeEncodeExpand::decode(&encoded)?;

    assert_eq!(decoded.fixed_a, value.fixed_a);
    assert_eq!(decoded.fixed_b, value.fixed_b);
    assert_eq!(decoded.var1_a, value.var1_a);
    assert_eq!(decoded.var1_c, value.var1_c.as_slice());
    assert_eq!(decoded.fixed_c, value.fixed_c);
    assert_eq!(decoded.var1_b, value.var1_b);
    assert_eq!(decoded.fixed_d, value.fixed_d);
    assert_eq!(decoded.var2.len(), value.var2.len());
    for (decoded_item, expected) in decoded.var2.iter().zip(value.var2.iter()) {
        assert_eq!(*decoded_item, expected.as_slice());
    }
    println!("decoded ok: {decoded:?}");

    Ok(())
}
