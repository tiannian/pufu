# Field Encode

## Overview
`Encode` defines how a value contributes to the three buffers that make
up the binary framing used throughout the workspace: fixed, data, and
var_length. It is the contract every serialized field obeys, ensuring
primitive, array, and collection types write their payloads consistently
without leaking framing details to callers.

## Detailed Specifications

### Encode trait
- Signature:
  ```rust
  pub trait Encode {
      fn encode_field<const IS_LAST_VAR: bool>(&self, e: &mut Encoder);
  }
  ```
- Every implementation receives a mutable `Encoder` carrying little/big
  endian helpers plus buffers for `fixed`, `data`, and `var_length` regions.
- The caller chooses `IS_LAST_VAR` to indicate whether the field being
  encoded is the final variable-length field for the enclosing struct;
  implementations must respect this flag when writing var-length data.

### Fixed-length primitive and array data
- All fixed-width primitives (`u8`, `i8`, `u16`, `i16`, …, `usize`,
  `isize`) use `push_fixed_data(&mut e.fixed, &e.endian)` so the bytes land
  in the `fixed` buffer.
- Fixed-size arrays (`[T; N]`) and their references also use
  `push_fixed_data` to consume `N * T::LENGTH` bytes, so consumers need not
  distinguish between owned and borrowed array instances.

### Variable-length semantics via DataType::MODE
- Every element type exposes `DataMode` (from `core/src/encode.rs` via
  `DataType`) which is either `Fixed` or `Var1`.
- When encoding a `Vec<T>` or slice:
  - If `T::MODE == DataMode::Fixed`, the implementation appends the
    serialized bytes to `e.data` and records the total byte length in
    `e.var_length` as a `u32`. This keeps the resulting frame aware of
    how much dynamic data follows.
  - If `T::MODE == DataMode::Var1`, the collection is treated as a
    var-length list of var-length values (a so-called “var2” field);
    each element writes length metadata into `e.var_length` before
    appending bytes to `e.data` via `push_var1_data`.
- Borrowed vectors/slices just forward to the owned `Vec<T>` behavior so
  callers can mix references without duplicating logic.

### Last-variable restriction
- Writing `DataMode::Var1` content is only permitted when
  `IS_LAST_VAR == true`. The const-generic parameter makes this check
  manifest without runtime cost.
- Attempting to encode a var2-style collection when `IS_LAST_VAR == false`
  triggers a panic with the message `"var2 vectors cannot be encoded as
  last variable field"`, preventing mis-framing of subsequent fields.

## References
- `core/src/encode.rs` (Encode implementations and Encoder buffers)
