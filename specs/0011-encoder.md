# Encoder for Binary Serialization Protocol

## Overview

This specification defines the **Encoder** type used to build binary payloads that conform to the wire layout described in `0010-binary-serde.md`. The Encoder accumulates a fixed-size region, a variable-entry index (data-relative offsets during construction), and a variable-length data region, then writes a complete payload on finalization. The Encoder does **not** write magic or version; the payload is the layout body only (see finalize).

## Detailed Specifications

### Encoder Structure

| Field    | Type     | Description                                                                 |
|----------|----------|-----------------------------------------------------------------------------|
| `fixed`  | `Vec<u8>`| Bytes for the FixedRegion; appended in schema order.                       |
| `var_idx`| `Vec<u32>`| During encoding: data-relative offsets (indices into `data`). On finalize, these are converted to payload-start-relative offsets for the VarEntry region. |
| `data`   | `Vec<u8>`| Concatenated variable-length values in schema order.                        |

Invariants: `var_idx` entries must be in the range `[0, data.len()]` and non-decreasing when used as start offsets for variable-length values. The number of `var_idx` entries must match the number of variable-length fields defined by the schema.

---

### Constructor

- **`new() -> Self`**  
  Returns an empty Encoder with empty `fixed`, `var_idx`, and `data`.

---

### Methods

- **`push_fixed(&mut self, bytes: &[u8])`**  
  Appends `bytes` to `fixed`. Used for fixed-length fields (primitives or fixed-size byte arrays) in schema order.

- **`push_var_idx(&mut self, offset: u32)`**  
  Pushes a single `u32` value onto `var_idx`. The value is a data-relative offset (e.g. `data.len() as u32` at the time the corresponding variable-length value starts). Caller is responsible for ensuring the offset is valid.

- **`push_data(&mut self, bytes: &[u8])`**  
  Appends `bytes` to `data`. Used for variable-length values (e.g. `Bytes`, `String`, or raw segments). Use `push_var_idx` to record the start offset when needed.

- **`finalize(self, out: &mut Vec<u8>) -> Result<(), CodecError>`**  
  Consumes the Encoder and writes the payload into `out` (appends). Does **not** write magic or version. Layout: `total_len` (u32), `var_entry_offset` (u32), `data_offset` (u32), FixedRegion, VarEntry (each `u32` converted to payload-start-relative offset), Data. All offsets are relative to the first byte of this payload. Returns `Ok(())` on success; may return `CodecError` if the Encoder state is invalid (e.g. offset overflow). Byte order for multi-byte fields must match the decoder (e.g. little-endian).

---

## References

- Wire layout and header semantics: `0010-binary-serde.md`.
- Error type: `CodecError` in the core codec module.
