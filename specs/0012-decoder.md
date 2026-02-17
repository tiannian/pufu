# Decoder for Binary Deserialization Protocol

## Overview

This specification defines the **Decoder** type used to read binary payloads that conform to the wire layout described in `0010-binary-serde.md`. The Decoder parses a payload by maintaining cursors for the fixed region and variable-entry index, and provides methods to read fixed-length bytes and variable-length values from their respective regions.

## Detailed Specifications

### Decoder Structure

| Field            | Type      | Description                                                                 |
|------------------|-----------|-----------------------------------------------------------------------------|
| `buf`            | `&[u8]`   | Reference to the complete payload buffer to decode. The first bytes must contain the payload header written by the Encoder. |
| `total_len`      | `u32`     | Total payload length in bytes, as read from the header.                    |
| `var_idx_offset` | `u32`     | Offset within the payload where the VarEntry region starts (relative to the payload start). |
| `data_offset`    | `u32`     | Offset within the payload where the Data region starts (relative to the payload start). |
| `fixed_cursor`   | `u32`     | Current read position within the FixedRegion (relative to the start of the FixedRegion). |
| `var_cursor`     | `u32`     | Current read position within the VarEntry region (relative to `var_idx_offset`). |

Invariants: `total_len` must be less than or equal to `buf.len()`. `var_idx_offset` and `data_offset` must be valid offsets within the payload, with `var_idx_offset >= 12` (header length), `data_offset >= var_idx_offset`, and `data_offset <= total_len`. `fixed_cursor` must not exceed the size of the FixedRegion. `var_cursor` must not exceed the size of the VarEntry region.

---

### Constructor

- **`new(buf: &[u8]) -> Result<Self, CodecError>`**  
  Creates a Decoder by parsing the header from `buf`. Expects the first 12 bytes to be `total_len`, `var_idx_offset`, and `data_offset` (each `u32` LE). Validates that these values describe a payload fully contained within `buf`. On success, returns a Decoder with `fixed_cursor` and `var_cursor` set to 0; otherwise returns `CodecError::InvalidLength`.

---

### Methods

- **`var_count(&self) -> u32`**  
  Returns the number of variable-length entries. Calculated as `(data_offset - var_idx_offset) / 4`. This represents the number of `u32` entries in the VarEntry region.

- **`next_fixed_bytes(&mut self, len: u32) -> Result<&[u8], CodecError>`**  
  Reads the next `len` bytes from the FixedRegion starting at `fixed_cursor`. Advances `fixed_cursor` by `len`. Returns a slice of `buf` containing the bytes, or `CodecError` if there are insufficient bytes remaining in the FixedRegion or if the read would exceed buffer bounds.

- **`next_var(&self, idx: u32) -> Result<&[u8], CodecError>`**  
  Reads the `idx`-th variable-length value. Interprets the `idx`-th VarEntry `u32` as an absolute payload offset into the Data region. For all but the last entry, uses the `(idx + 1)`-th offset as the end of the slice; for the last entry, uses `total_len` as the end. Returns a slice of `buf` for the computed range, or `CodecError` if indices or offsets are invalid or out of bounds.

---

## References

- Wire layout and header semantics: `0010-binary-serde.md`.
- Encoder specification: `0011-encoder.md`.
- Error type: `CodecError` in the core codec module.
