# Codec Trait for Zero-Copy Encode/Decode

## Overview

This specification describes the `Codec` trait and its error type used for zero-copy encode and decode operations. Types implementing `Codec` can be encoded into a byte vector and decoded into an optional zero-copy view over a buffer, with support for validation without constructing the view.

## Detailed Specifications

### Error Type: `CodecError`

Error type for codec operations (validation or decode failure).

| Variant            | Description                                                                 |
|--------------------|-----------------------------------------------------------------------------|
| `InvalidLength`    | Buffer length or layout is invalid.                                        |
| `ValidationFailed` | Data failed validation (e.g. checksum, magic, or structural check).        |
| `Message(String)`  | Custom message for diagnostics.                                            |

`CodecError` implements `Debug`, `Clone`, `PartialEq`, `Eq`, `Display`, and `std::error::Error`. Display text for the variants is: `"invalid length"`, `"validation failed"`, or the custom string for `Message`.

---

### Trait: `Codec`

Trait for types that can be encoded and decoded with an optional zero-copy view.

**Associated type**:

- `View<'a>` — zero-copy view type; must be `Sized` and tied to the lifetime `'a` of the buffer, with `Self: 'a`.

**Required methods**:

| Method | Signature | Description |
|--------|-----------|-------------|
| `encode` | `fn encode(&self, buf: &mut Vec<u8>)` | Append the encoded value to the given byte vector. |
| `decode` | `fn decode<'a>(buf: &'a [u8]) -> Result<Self::View<'a>, CodecError>` | Decode the buffer into a zero-copy view. |
| `validate` | `fn validate(buf: &[u8]) -> Result<(), CodecError>` | Optionally validate the buffer without constructing the view. |

**Contract**:

- `encode(self, buf)` appends the encoded byte sequence to `buf`; that sequence, when passed to `decode`, yields a view equivalent to the original value (for the intended semantics of the type).
- `validate(buf)` succeeds if and only if `decode(buf)` would succeed; it may be used to check layout or integrity without allocating or building the view.
- Decode and validate must report `CodecError::InvalidLength` when the buffer length or layout is invalid, and `CodecError::ValidationFailed` when a checksum, magic, or structural check fails.

---

### Relation to Other Specs

- The wire layout and binary format are defined in `0010-binary-serde.md`; the trait layout taxonomy (e.g. fixed vs variable-length) is in `0011-trait-layout-taxonomy.md`. The `Codec` trait is a generic contract for types that support encode/decode/validate; concrete formats (including the binary protocol above) may be implemented on top of it or alongside it.

---

## References

- `core/src/codec.rs` — trait and `CodecError` implementation.
- `0010-binary-serde.md` — wire layout (Header, FixedRegion, VarEntry, Data).
- `0011-trait-layout-taxonomy.md` — type classification (ZcFixed, ZcVar1, ZcVar2).
