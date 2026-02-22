//! Integration tests for derive macros.

use pufu_core::{Decode as DecodeTrait, Decoder, Encode as EncodeTrait, Encoder};
use pufu_macros::{Decode, Encode};

/// Minimal payload with a var1 field.
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
/// Payload that includes a var2 field (Vec<Vec<T>>).
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

#[derive(Encode, Decode)]
/// Inner payload for nested-struct test.
struct InnerPayload {
    tag: u16,
    data: Vec<u8>,
}

#[derive(Encode, Decode)]
/// Outer payload that nests another derived struct.
struct OuterPayload {
    version: u8,
    inner: InnerPayload,
    tail: u32,
}

#[test]
fn derive_encode_decode_roundtrip_nested() {
    let value = OuterPayload {
        version: 1,
        inner: InnerPayload {
            tag: 0x0203,
            data: vec![0xde, 0xad, 0xbe, 0xef],
        },
        tail: 0x0a0b0c0d,
    };

    let mut encoder = Encoder::little();
    value.encode_field::<true>(&mut encoder);

    let mut out = Vec::new();
    encoder.finalize(&mut out);

    let mut decoder = Decoder::new(&out).expect("decoder");
    let view = OuterPayload::decode_field::<true>(&mut decoder).expect("view");

    assert_eq!(view.version, value.version);
    assert_eq!(view.inner.tag, value.inner.tag);
    assert_eq!(view.inner.data, value.inner.data.as_slice());
    assert_eq!(view.tail, value.tail);
}

#[derive(Encode, Decode)]
/// Fixture payload matching the core encode/expand test.
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

#[derive(Encode, Decode)]
/// Outer wrapper for the encode/expand fixture.
struct EncodeEncodeExpandOuter {
    prefix: u8,
    inner: EncodeEncodeExpand,
    suffix: Vec<u8>,
}

#[test]
fn derive_encode_matches_encode_encode_expand_fixture() {
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

    let mut encoder = Encoder::little();
    value.encode_field::<true>(&mut encoder);

    let mut out = Vec::new();
    encoder.finalize(&mut out);

    let expected = "5e000000080000000c000000520000000900000042110000004f0000003e000000170000000403020106050719181716151413122b0000003100000035000000390000003c0000000a0014001e00090807060b0a0d0c01020304050feedd";
    assert_eq!(hex::encode(&out), expected);
}
