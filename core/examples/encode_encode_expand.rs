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
    fixed_a: <u32 as Decode>::View<'a>,
    fixed_b: <u16 as Decode>::View<'a>,
    var1_a: <Vec<u16> as Decode>::View<'a>,
    var1_c: <Vec<u8> as Decode>::View<'a>,
    fixed_c: <u8 as Decode>::View<'a>,
    var1_b: <Vec<u16> as Decode>::View<'a>,
    fixed_d: <u64 as Decode>::View<'a>,
    var2: <Vec<Vec<u8>> as Decode>::View<'a>,
}

#[derive(Debug, PartialEq, Eq)]
struct EncodeEncodeExpandOuter {
    prefix: u8,
    inner: EncodeEncodeExpand,
    suffix: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
struct EncodeEncodeExpandOuterView<'a> {
    prefix: <u8 as Decode>::View<'a>,
    inner: <EncodeEncodeExpand as Decode>::View<'a>,
    suffix: <Vec<u8> as Decode>::View<'a>,
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
        let mut nested_encoder = Encoder::new(encoder.endian);
        self.fixed_a.encode_field::<false>(&mut nested_encoder);
        self.fixed_b.encode_field::<false>(&mut nested_encoder);
        self.var1_a.encode_field::<false>(&mut nested_encoder);
        self.var1_c.encode_field::<false>(&mut nested_encoder);
        self.fixed_c.encode_field::<false>(&mut nested_encoder);
        self.var1_b.encode_field::<false>(&mut nested_encoder);
        self.fixed_d.encode_field::<false>(&mut nested_encoder);
        self.var2.encode_field::<true>(&mut nested_encoder);

        let mut nested_payload = Vec::new();
        nested_encoder.finalize(&mut nested_payload);
        nested_payload.encode_field::<IS_LAST_VAR>(encoder);
    }
}

impl Decode for EncodeEncodeExpand {
    type View<'a> = EncodeEncodeExpandView<'a>;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError> {
        let nested_payload = Vec::<u8>::decode_field::<IS_LAST_VAR>(decoder)?;
        let mut nested_decoder = Decoder::new(nested_payload)?;

        let fixed_a = u32::decode_field::<false>(&mut nested_decoder)?;
        let fixed_b = u16::decode_field::<false>(&mut nested_decoder)?;
        let var1_a = Vec::<u16>::decode_field::<false>(&mut nested_decoder)?;
        let var1_c = Vec::<u8>::decode_field::<false>(&mut nested_decoder)?;
        let fixed_c = u8::decode_field::<false>(&mut nested_decoder)?;
        let var1_b = Vec::<u16>::decode_field::<false>(&mut nested_decoder)?;
        let fixed_d = u64::decode_field::<false>(&mut nested_decoder)?;
        let var2 = Vec::<Vec<u8>>::decode_field::<true>(&mut nested_decoder)?;

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

impl EncodeEncodeExpandOuter {
    fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::little();

        self.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        out
    }

    fn decode(buf: &[u8]) -> Result<EncodeEncodeExpandOuterView<'_>, CodecError> {
        let mut decoder = Decoder::new(buf)?;

        Self::decode_field::<true>(&mut decoder)
    }
}

impl Encode for EncodeEncodeExpandOuter {
    fn encode_field<const IS_LAST_VAR: bool>(&self, encoder: &mut Encoder) {
        let mut nested_encoder = Encoder::new(encoder.endian);
        self.prefix.encode_field::<false>(&mut nested_encoder);
        self.inner.encode_field::<false>(&mut nested_encoder);
        self.suffix.encode_field::<true>(&mut nested_encoder);

        let mut nested_payload = Vec::new();
        nested_encoder.finalize(&mut nested_payload);
        nested_payload.encode_field::<IS_LAST_VAR>(encoder);
    }
}

impl Decode for EncodeEncodeExpandOuter {
    type View<'a> = EncodeEncodeExpandOuterView<'a>;

    fn decode_field<'a, const IS_LAST_VAR: bool>(
        decoder: &mut Decoder<'a>,
    ) -> Result<Self::View<'a>, CodecError> {
        let nested_payload = Vec::<u8>::decode_field::<IS_LAST_VAR>(decoder)?;
        let mut nested_decoder = Decoder::new(nested_payload)?;

        let prefix = u8::decode_field::<false>(&mut nested_decoder)?;
        let inner = EncodeEncodeExpand::decode_field::<false>(&mut nested_decoder)?;
        let suffix = Vec::<u8>::decode_field::<true>(&mut nested_decoder)?;

        Ok(EncodeEncodeExpandOuterView {
            prefix,
            inner,
            suffix,
        })
    }
}

fn main() -> Result<(), CodecError> {
    let inner = EncodeEncodeExpand {
        fixed_a: 0x0102_0304,
        fixed_b: 0x0506,
        var1_a: vec![10, 20, 30],
        var1_c: vec![9, 8, 7, 6],
        fixed_c: 0x07,
        var1_b: vec![0x0a0b, 0x0c0d],
        fixed_d: 0x1213_1415_1617_1819,
        var2: vec![vec![1, 2, 3], vec![4, 5]],
    };

    let value = EncodeEncodeExpandOuter {
        prefix: 0x42,
        inner,
        suffix: vec![0x0f, 0xee, 0xdd],
    };

    let encoded = value.encode();
    let decoded = EncodeEncodeExpandOuter::decode(&encoded)?;

    assert_eq!(decoded.prefix, value.prefix);
    assert_eq!(decoded.inner.fixed_a, value.inner.fixed_a);
    assert_eq!(decoded.inner.fixed_b, value.inner.fixed_b);
    assert_eq!(decoded.inner.var1_a, value.inner.var1_a);
    assert_eq!(decoded.inner.var1_c, value.inner.var1_c.as_slice());
    assert_eq!(decoded.inner.fixed_c, value.inner.fixed_c);
    assert_eq!(decoded.inner.var1_b, value.inner.var1_b);
    assert_eq!(decoded.inner.fixed_d, value.inner.fixed_d);
    assert_eq!(decoded.inner.var2.len(), value.inner.var2.len());
    for (decoded_item, expected) in decoded.inner.var2.iter().zip(value.inner.var2.iter()) {
        assert_eq!(*decoded_item, expected.as_slice());
    }
    assert_eq!(decoded.suffix, value.suffix.as_slice());
    println!("decoded ok: {decoded:?}");

    Ok(())
}
