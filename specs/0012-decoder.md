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
  Returns the number of variable-length entries. Calculated as `(data_offset - var_idx_offset) / 4`, matching the length of `var_idx` that the encoder wrote. Decoder consumers, especially `FieldDecode` implementations, must use this count to verify they do not read beyond the available var entries.

- **`next_fixed_bytes(&mut self, len: u32) -> Result<&[u8], CodecError>`**  
  Reads the next `len` bytes from the FixedRegion starting at `fixed_cursor`. Advances `fixed_cursor` by `len`. Returns a slice of `buf` containing the bytes, or `CodecError::InvalidLength` if there are insufficient bytes remaining in the FixedRegion, the read would exceed `data_offset`, or the length is larger than the remaining fixed region budget. Fixed-length fields decoded via `FieldDecode` must match the exact lengths written by `FieldEncode`.

- **`get_var(&self, idx: u32) -> Result<&[u8], CodecError>`**  
  Reads the `idx`-th variable-length entry slice. Interprets the `idx`-th VarEntry `u32` as a payload-relative offset into the Data region and uses the next entry (or `total_len` for the final entry) to compute the end offset. Performs bounds checks: the start offset must be at least `data_offset`, not exceed `total_len`, and must not overflow when translated into a slice. Returns a borrowed slice of the Data region; `FieldDecode` implementations consume this slice to reconstruct variable-length fields, including var1/var2 values described in `specs/0016-field-decode.md`. The returned slice lives for `'a`, so callers must not outlive the decoder buffer.

---

## References

- Wire layout and header semantics: `0010-binary-serde.md`.
- Encoder specification: `0011-encoder.md`.
- Error type: `CodecError` in the core codec module.
