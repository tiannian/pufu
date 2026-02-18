# Derive Macros for Encode and Decode

## Overview
This specification defines a derive macro that generates `Encode` and `Decode` trait
implementations for structs by following the same field order and framing rules
used in manual implementations such as `core/examples/encode_encode_expand.rs`.
The macro removes boilerplate while preserving the encoding layout and var-length
constraints defined by the existing field-level specs.

## Detailed Specifications

### Macro name and scope
- The derive macro is named `Codec` and is invoked as `#[derive(Codec)]`.
- The macro generates both `Encode` and `Decode` implementations for the target
  struct.
- Only structs are supported. Enums and unions must be rejected at macro-expansion
  time with a clear error message.

### Supported struct forms
- Named-field structs are supported.
- Tuple structs and unit structs are rejected with a clear error message.
- All fields are encoded/decoded in declaration order.

### Generated `Encode` implementation
- Signature:
  ```rust
  impl Encode for <Struct> {
      fn encode_field<const IS_LAST_VAR: bool>(&self, encoder: &mut Encoder) {
          let _ = IS_LAST_VAR;
          // field encoding in order
      }
  }
  ```
- For each field, the macro generates a call of the form:
  ```rust
  self.<field>.encode_field::<FLAG>(encoder);
  ```
- `FLAG` is computed based on the “last variable field” rules below. All other
  fields receive `false`.

### Generated `Decode` implementation
- Signature:
  ```rust
  impl Decode for <Struct> {
      type View<'a> = <Struct>View<'a>;

      fn decode_field<'a, const IS_LAST_VAR: bool>(
          decoder: &mut Decoder<'a>,
      ) -> Result<Self::View<'a>, CodecError> {
          let _ = IS_LAST_VAR;
          // field decoding in order
      }
  }
  ```
- The macro generates a companion view struct named `<Struct>View<'a>` in the
  same scope as the target struct.
- `<Struct>View<'a>` mirrors the original field names and order, and each field
  type is written using the original type’s `Decode::View` associated type:
  `field: <FieldType as Decode>::View<'a>`.
- Example mappings (as written in the generated view struct):
  - `var1_a: <Vec<u16> as Decode>::View<'a>` (decodes to `Vec<u16>`)
  - `var1_c: <Vec<u8> as Decode>::View<'a>` (decodes to `&'a [u8]`)
  - `var2: <Vec<Vec<u8>> as Decode>::View<'a>` (decodes to `Vec<&'a [u8]>`)
- For each field, the macro generates a call of the form:
  ```rust
  let <field> = <FieldType>::decode_field::<FLAG>(decoder)?;
  ```
- The resulting struct is constructed with the decoded field values in the same
  order as declared.

### Last-variable field selection
- The derive macro must identify variable-length fields by syntax:
  - **Var1 fields**: `Vec<T>` where `T` is not `Vec<...>`.
  - **Var2 fields**: `Vec<Vec<T>>`.
- The **last variable field** is the final field in declaration order that is
  either a Var1 or Var2 field. This field receives `FLAG = true`.
- All other fields receive `FLAG = false`.
- If there are **no** variable-length fields, the macro sets `FLAG = true` on
  the last field for determinism; all other fields receive `false`.
- Var2 fields **must** be the last variable field. If a Var2 field appears
  before another variable-length field, the macro must emit a compile-time error
  indicating that var2 data must be encoded/decoded as the last variable field.
- Fixed-length fields may appear after the last variable field; they still use
  `FLAG = false` because they do not consume var-length entries.

### Trait bounds and generics
- The generated implementations must preserve the struct’s generics and where
  clauses.
- For each field type `T`, the macro adds trait bounds as needed so that:
  - `T: Encode` is available for the generated `Encode` implementation.
  - `T: Decode` is available for the generated `Decode` implementation.

### Error handling behavior
- The generated `Decode` implementation propagates `CodecError` exactly as the
  manual implementation would, by forwarding errors from `decode_field` calls.
- The macro does not introduce additional runtime checks beyond the var2
  placement restriction; it relies on `Encode`/`Decode` implementations to
  enforce framing constraints.

## References
- Example manual implementation: `core/examples/encode_encode_expand.rs`
- Field encode rules: `specs/0015-field-encode.md`
- Field decode rules: `specs/0016-field-decode.md`
- Encoder framing: `specs/0011-encoder.md`
- Decoder framing: `specs/0012-decoder.md`
