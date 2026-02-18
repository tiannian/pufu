use pufu_core::{Decode as DecodeTrait, Decoder, Encode as EncodeTrait, Encoder};
use pufu_macros::{Decode, Encode};

#[derive(Encode, Decode)]
struct SimplePayload {
    id: u16,
    payload: Vec<u8>,
}

#[test]
fn derive_encode_decode_roundtrip_var1() {
    let value = SimplePayload {
        id: 0x1234,
        payload: vec![0xaa, 0xbb, 0xcc],
    };

    let mut encoder = Encoder::little();
    value.encode_field::<true>(&mut encoder);

    let mut out = Vec::new();
    encoder.finalize(&mut out);

    let mut decoder = Decoder::new(&out).expect("decoder");
    let view = SimplePayload::decode_field::<true>(&mut decoder).expect("view");

    assert_eq!(view.id, value.id);
    assert_eq!(view.payload, value.payload.as_slice());
}

#[derive(Encode, Decode)]
struct NestedPayload {
    fixed: u8,
    payload: Vec<u8>,
    nested: Vec<Vec<u16>>,
}

#[test]
fn derive_encode_decode_roundtrip_var2_last() {
    let value = NestedPayload {
        fixed: 0x7f,
        payload: vec![0x10, 0x20],
        nested: vec![vec![1, 2], vec![3]],
    };

    let mut encoder = Encoder::little();
    value.encode_field::<true>(&mut encoder);

    let mut out = Vec::new();
    encoder.finalize(&mut out);

    let mut decoder = Decoder::new(&out).expect("decoder");
    let view = NestedPayload::decode_field::<true>(&mut decoder).expect("view");

    assert_eq!(view.fixed, value.fixed);
    assert_eq!(view.payload, value.payload.as_slice());
    assert_eq!(view.nested, value.nested);
}
