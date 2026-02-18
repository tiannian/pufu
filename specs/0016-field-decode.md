# Field Decode

## Overview
`Decode` describes how individual fields consume bytes from the binary framing laid out by the Encoder/Decoder pair. Whereas `Encode` pushes values into the fixed, data, and var_length regions, field-level decoding must interpret the same regions and expose either borrowed views or owned values without reinterpreting the framing details.

## Detailed Specifications

### Decode trait
- Signature:
  ```rust
  pub trait Decode {
      type View<'a>
      where
          Self: 'a;

      fn decode_field<'a, const IS_LAST_VAR: bool>(
          decoder: &mut Decoder<'a>,
      ) -> Result<Self::View<'a>, CodecError>;
  }
  ```
- The decoder passed to `decode_field` is expected to already have parsed the payload header (`total_len`, `var_idx_offset`, `data_offset`) and implements the cursor helpers (`next_fixed_bytes`, `var_count`, `next_var`).
- Associated `View<'a>` binds the returned type to the lifetime of the decoder buffer so borrowed variants can safely point into the payload without cloning.

### View<'a> contract
- Implementations decide whether `View<'a>` is a fully owned value (e.g., `Vec<T>`) or a borrowed reference (e.g., `&'a T`, `&'a [T]`, `Cow<'a, T>`).
- The lifetime `'a` must be tied to the decoder buffer, and implementations must avoid keeping references that outlive the decoder or the underlying payload slice.
- `View` should expose the decoded value in the most ergonomic form for consumers while minimizing copies; when borrowing is sufficient, prefer references to the slices returned by `Decoder::next_fixed_bytes` and `Decoder::next_var`.

### decode_field semantics
- `decode_field` returns `Result<Self::View<'a>, CodecError>` to propagate framing errors reported by the decoder helpers.
- Implementations consume bytes in the same order as the fields were encoded. The surrounding deserializer is responsible for advancing through the fixed region and for tracking which variable-length entry index is next; `decode_field` itself only reads exactly the bytes it needs.
- The const-generic `IS_LAST_VAR` mirrors the encoder so implementations can enforce var2 restrictions. When `IS_LAST_VAR` is `true`, the caller is signaling that no further variable-length fields follow, allowing `DataMode::Var1` values to read the remaining var entries without spilling into subsequent fields.

### Fixed-length data
- For `DataMode::Fixed` values (primitives, fixed-size arrays) implementations must call `decoder.next_fixed_bytes(<length>)`, where `<length>` is the number of bytes produced by `Encode` (`T::LENGTH` for `T: DataType`).
- The returned slice may be interpreted directly (e.g., via `from_le_bytes`) or reborrowed for types that expose references into the payload.
- Fixed-length decoding must maintain the decoder cursor order: after reading a fixed field, the next field sees the cursor advanced by the number of consumed bytes.

### Variable-length data
- Variable-length fields that produced `DataMode::Fixed` entries (e.g., `Vec<T>` with fixed elements) must fetch their data via `decoder.next_var()` using the next unconsumed variable index. That slice contains the concatenated fixed-size elements and the implementations are responsible for parsing it or reusing it as a borrowed slice.
- For `DataMode::Var1` values (var2 data), the implementation reads the remaining variable entries (each representing a var1 field) via repeated `decoder.next_var`. Because every var1 entry writes its own offset into `var_length`, the decode path reconstructs the list by slicing the returned data entries in order.
- Reading var2 data is only permitted when `IS_LAST_VAR == true`, mirroring the encoder panic guard. Attempting to decode a var2 field when additional variable fields follow must produce a `CodecError::InvalidLength` or similar guard that prevents mixing var1 entries belonging to different fields.

### Ordering and error handling
- The decoder guarantees that the var entry count (`Decoder::var_count`) matches the number of variable-length fields encoded; field decode implementations must not read beyond the available indices.
- Any violation—requesting more fixed bytes than remain, reading a var entry index that is out of bounds, or encountering inconsistent offsets—is reported through `CodecError` so callers can reject malformed payloads.
- Because `Decode` implementations borrow directly from the payload, they should not retain references after the decoder buffer is dropped, and the surrounding deserializer must ensure the decoder lives at least as long as any retained `View<'a>` objects.

## References
- Encoder framing and header layout: `specs/0011-encoder.md`, `core/src/encoder.rs`
- Decoder helpers and var-entry layout: `specs/0012-decoder.md`, `core/src/decoder.rs`
- Field encode expectations and var2 constraints: `specs/0015-field-encode.md`, `core/src/encode.rs`
- Data mode metadata that determines fixed vs. var1 behavior: `core/src/data_type.rs`
