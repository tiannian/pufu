# Decoder for Binary Deserialization Protocol

## Overview

This specification defines the **Decoder** type used to read binary payloads that conform to the wire layout described in `0010-binary-serde.md`. The Decoder is initialized with a **Config** (see `0017-config.md`) and parses a payload by maintaining cursors for the fixed region and the var-entry index, and provides helpers to read fixed-length bytes and variable-length values from their respective regions. Accessors `magic()`, `version()`, and `endian()` expose the config used for decoding.

## Detailed Specifications

### Decoder Structure

| Field            | Type      | Description                                                                 |
|------------------|-----------|-----------------------------------------------------------------------------|
| `config`         | `Config`  | Magic, version, and endianness from `0017-config.md`; used for validation and by accessors `magic()`, `version()`, `endian()`, and for reading multi-byte values. |
| `buf`            | `&[u8]`   | Reference to the complete payload buffer to decode. Bytes passed to the Decoder start immediately after the magic+version prefix described in `specs/0010-binary-serde.md`. |
| `total_len`      | `u32`     | Total payload length in bytes from the first byte of `total_len` through the end of the payload (excludes magic+version). |
| `var_idx_offset` | `u32`     | Offset from the start of this payload (i.e. the first byte after magic+version) where the VarEntry region begins. |
| `data_offset`    | `u32`     | Computed offset of the first byte of the Data region. When variable entries exist, this equals the first VarEntry offset; otherwise it equals `var_idx_offset`. |
| `fixed_cursor`   | `u32`     | Current read position within the FixedRegion (relative to the start of the FixedRegion). |
| `var_cursor`     | `u32`     | Current read index within the VarEntry region, used by `next_var`. |

Invariants: `total_len` must be less than or equal to `buf.len()`. `var_idx_offset` and `data_offset` must be valid offsets within the payload, with `var_idx_offset >= 8` (header length after dropping magic+version from `specs/0010-binary-serde.md`), `data_offset >= var_idx_offset`, and `data_offset <= total_len`. `fixed_cursor` must not exceed the size of the FixedRegion.

---

### Constructor

- **`new(buf: &[u8], config: Config) -> Result<Self, CodecError>`**  
  Creates a Decoder with the given **Config** (see `0017-config.md`) by parsing the 8-byte payload header that follows the magic+version prefix. The first two multi-byte values are `total_len` and `var_idx_offset`, read using the config’s endianness. The decoder then infers `data_offset`: if `total_len == var_idx_offset`, there are no VarEntry slots and `data_offset == var_idx_offset`; otherwise, the first VarEntry slot (at `var_idx_offset`) contains the payload-relative offset of the first data value and is promoted to `data_offset`. Validates that all offsets and regions stay inside `buf`, that the derived VarEntry region is aligned to `u32`, and that `data_offset` lies between `var_idx_offset` and `total_len`. On success, returns a Decoder holding `config` with `fixed_cursor` at the start of the FixedRegion and the computed `data_offset`; otherwise returns `CodecError::InvalidLength`.

---

### Methods

- **`magic(&self) -> [u8; 4]`**  
  Returns the 4-byte magic from the Decoder’s config (the same magic used to identify this payload format).

- **`version(&self) -> u8`**  
  Returns the protocol version from the Decoder’s config.

- **`endian(&self) -> &Endian`**  
  Returns a reference to the endianness from the Decoder’s config. Used when reading multi-byte header and VarEntry values.

- **`var_count(&self) -> u32`**  
  Returns the number of variable-length entries present in the VarEntry region. This is `(data_offset - var_idx_offset) / 4` once `data_offset` has been inferred. `Decode` implementations must not request indices greater than or equal to this value.

- **`next_fixed_bytes(&mut self, len: u32) -> Result<&[u8], CodecError>`**  
  Reads the next `len` bytes from the FixedRegion starting at `fixed_cursor`. Advances the cursor and returns a slice of `buf`. Returns `CodecError::InvalidLength` if `len` exceeds the remaining bytes before `var_idx_offset`, if arithmetic overflows occur, or if the resulting slice would stray outside the FixedRegion bounds. Fixed-length fields decoded via `Decode` must request exactly the lengths written by `Encode` so the cursor remains aligned with schema order.

- **`next_var(&mut self) -> Result<&[u8], CodecError>`**  
  Retrieves the next variable-length entry slice using `var_cursor`. Interprets the current VarEntry `u32` (per `specs/0010-binary-serde.md`) as an absolute payload offset into the Data region and uses the next VarEntry offset (or `total_len` for the final entry) to compute the end, then advances `var_cursor` by one. Performs bounds checks: the start must be at least `data_offset`, offsets must not decrease, and the final range must stay inside `[data_offset, total_len]` and `buf.len()`. Returns a borrowed slice tied to `'a`; `Decode` implementations consume it to rebuild var1/var2 payloads without copying when possible.

---

## References

- Config and ConfigBuilder: `0017-config.md`.
- Wire layout and header semantics: `0010-binary-serde.md`.
- Encoder specification: `0011-encoder.md`.
- Error type: `CodecError` in the core codec module.
