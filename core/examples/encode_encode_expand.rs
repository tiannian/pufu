use pufu_core::{CodecError, Decoder, Encoder, FieldDecode, FieldEncode};

#[derive(Debug, PartialEq, Eq)]
struct EncodeEncodeExpand {
    fixed_a: u32,
    fixed_b: u16,
    var1_a: Vec<u16>,
    fixed_c: u8,
    var1_b: Vec<u32>,
    fixed_d: u64,
    var2: Vec<Vec<u8>>,
}

impl EncodeEncodeExpand {
    fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::little();

        self.fixed_a.encode_field::<false>(&mut encoder);
        self.fixed_b.encode_field::<false>(&mut encoder);
        self.var1_a.encode_field::<false>(&mut encoder);
        self.fixed_c.encode_field::<false>(&mut encoder);
        self.var1_b.encode_field::<false>(&mut encoder);
        self.fixed_d.encode_field::<false>(&mut encoder);
        self.var2.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        out
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let mut decoder = Decoder::new(buf)?;

        let fixed_a = u32::decode_field::<false>(&mut decoder)?;
        let fixed_b = u16::decode_field::<false>(&mut decoder)?;
        let var1_a = Vec::<u16>::decode_field::<false>(&mut decoder)?;
        let fixed_c = u8::decode_field::<false>(&mut decoder)?;
        let var1_b = Vec::<u32>::decode_field::<false>(&mut decoder)?;
        let fixed_d = u64::decode_field::<false>(&mut decoder)?;
        let var2 = Vec::<Vec<u8>>::decode_field::<true>(&mut decoder)?;

        Ok(Self {
            fixed_a,
            fixed_b,
            var1_a,
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
        fixed_c: 0x07,
        var1_b: vec![0x0a0b_0c0d, 0x0e0f_1011],
        fixed_d: 0x1213_1415_1617_1819,
        var2: vec![vec![1, 2, 3], vec![4, 5]],
    };

    let encoded = value.encode();
    let decoded = EncodeEncodeExpand::decode(&encoded)?;

    assert_eq!(decoded, value);
    println!("decoded ok: {decoded:?}");

    Ok(())
}
