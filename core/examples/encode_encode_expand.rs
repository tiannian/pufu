use pufu_core::{CodecError, Decoder, Encoder, FieldDecode, FieldEncode};

#[derive(Debug, PartialEq, Eq)]
struct EncodeEncodeExpand {
    fixed: u32,
    var1: Vec<u16>,
    var2: Vec<Vec<u8>>,
}

impl EncodeEncodeExpand {
    fn encode(&self) -> Vec<u8> {
        let mut encoder = Encoder::little();

        self.fixed.encode_field::<false>(&mut encoder);
        self.var1.encode_field::<false>(&mut encoder);
        self.var2.encode_field::<true>(&mut encoder);

        let mut out = Vec::new();
        encoder.finalize(&mut out);
        out
    }

    fn decode(buf: &[u8]) -> Result<Self, CodecError> {
        let mut decoder = Decoder::new(buf)?;

        let fixed = u32::decode_field::<false>(&mut decoder)?;
        let var1 = Vec::<u16>::decode_field::<false>(&mut decoder)?;
        let var2 = Vec::<Vec<u8>>::decode_field::<true>(&mut decoder)?;

        Ok(Self { fixed, var1, var2 })
    }
}

fn main() -> Result<(), CodecError> {
    let value = EncodeEncodeExpand {
        fixed: 0x0102_0304,
        var1: vec![10, 20, 30],
        var2: vec![vec![1, 2, 3], vec![4, 5]],
    };

    let encoded = value.encode();
    let decoded = EncodeEncodeExpand::decode(&encoded)?;

    assert_eq!(decoded, value);
    println!("decoded ok: {decoded:?}");

    Ok(())
}
