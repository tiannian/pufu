# Encoder for Binary Serialization Protocol

## Overview

This specification defines the **Encoder** type used to build binary payloads that conform to the wire layout described in `0010-binary-serde.md`. The Encoder is initialized with a **Config** (see `0017-config.md`) and accumulates a fixed-size region, a variable-entry index (data-relative offsets during construction), and a variable-length data region, then writes a complete payload on finalization. The Encoder may write the layout body only (see `finalize`) or the full header including magic and version (see `finalize_with_magic_version`).

## Detailed Specifications

### Encoder Structure

| Field    | Type     | Description                                                                 |
|----------|----------|-----------------------------------------------------------------------------|
| `config` | `Config` | Magic, version, and endianness from `0017-config.md`; used by `finalize_with_magic_version` and for byte order when writing. |
| `fixed`  | `Vec<u8>`| Bytes for the FixedRegion; appended in schema order.                       |
| `var_idx`| `Vec<u32>`| During encoding: data-relative offsets (indices into `data`). On finalize, these are converted to payload-start-relative offsets for the VarEntry region. |
| `data`   | `Vec<u8>`| Concatenated variable-length values in schema order.                        |

Invariants: `var_idx` entries must be in the range `[0, data.len()]` and non-decreasing when used as start offsets for variable-length values. The number of `var_idx` entries must match the number of variable-length fields defined by the schema.

---

### Constructor

- **`new(config: Config) -> Self`**  
  Returns an Encoder with the given **Config** (see `0017-config.md`) and empty `fixed`, `var_idx`, and `data`. The config is used for magic, version, and endianness when calling `finalize_with_magic_version` and for writing multi-byte values.

---

### Methods

- **`push_fixed(&mut self, bytes: &[u8])`**  
  Appends `bytes` to `fixed`. Used for fixed-length fields (primitives or fixed-size byte arrays) in schema order.

- **`push_var_idx(&mut self, offset: u32)`**  
  Pushes a single `u32` value onto `var_idx`. The value is a data-relative offset (e.g. `data.len() as u32` at the time the corresponding variable-length value starts). Caller is responsible for ensuring the offset is valid.

- **`push_data(&mut self, bytes: &[u8])`**  
  Appends `bytes` to `data`. Used for variable-length values (e.g. `Bytes`, `String`, or raw segments). Use `push_var_idx` to record the start offset when needed.

- **`finalize(self, out: &mut Vec<u8>) -> Result<(), CodecError>`**  
  Consumes the Encoder and writes the payload into `out` (appends). Does **not** write magic or version. The method follows the implementation:

  * Compute the fixed `HEADER_LEN` of 12 bytes (three `u32` fields).  
  * Compute `var_entry_len` as `var_idx.len() * 4` and use it with `fixed.len()` and `data.len()` to derive `total_len`.  
  * Convert `total_len`, `var_entry_offset` (`HEADER_LEN + fixed.len()`), and `data_offset` (`var_entry_offset + var_entry_len`) into `u32`, returning `CodecError::InvalidLength` if any conversion overflows.  
  * Reserve `total_len` bytes in `out` and append the three header fields using the Encoder’s config endianness.  
  * Append `fixed`.  
  * For each `offset` in `var_idx`, add `data_offset` to get the payload-relative value and append it as `u32` in config endianness, returning `CodecError::InvalidLength` if the addition overflows.  
  * Append `data`.  
  
  Returns `Ok(())` when all steps succeed.

- **`finalize_with_magic_version(self, out: &mut Vec<u8>) -> Result<(), CodecError>`**  
  Consumes the Encoder and writes the **full** payload into `out` (appends): first the 4-byte **magic** and 1-byte **version** from the Encoder’s config, then the same layout produced by `finalize` (header fields `total_len`, `var_entry_offset`, FixedRegion, VarEntry, Data), with multi-byte fields written in config endianness. The stored `total_len` and `var_entry_offset` exclude the 5-byte magic+version prefix, consistent with `0010-binary-serde.md`. Returns `Ok(())` on success or `CodecError` on overflow/invalid length.  

---

## References

- Config and ConfigBuilder: `0017-config.md`.
- Wire layout and header semantics: `0010-binary-serde.md`.
- Error type: `CodecError` in the core codec module.
