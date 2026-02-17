# Binary and Bytes Serialization Protocol

## Overview

This specification defines a simplified binary serialization protocol for encoding structured data as bytes. The protocol uses a fixed layout with a header, a fixed-size region, a variable-entry index, and a variable-length data region. Schema is determined by code; no type information is embedded in the wire format.

## Detailed Specifications

### Wire Layout

The serialized payload has the following structure:

```text
[Header][FixedRegion][VarEntry][Data]
```

All offsets in the protocol are computed from the first byte of the magic (start of the payload).

---

### Header

The header is fixed-size and appears at the beginning of the payload.

| Field             | Type    | Description                                                                 |
|-------------------|---------|-----------------------------------------------------------------------------|
| `magic`           | 4 bytes | Magic identifier; the four bytes `b"svsd"` (0x73, 0x76, 0x73, 0x64).        |
| `version`         | u8      | Protocol version.                                                           |
| `total_len`       | u32     | Total length in bytes from the first byte of this field to the end of the payload; does **not** include the 4-byte magic or the 1-byte version. |
| `var_entry_offset`| u32     | Offset from the **first byte after magic and version** to the first byte of the VarEntry region. Stored value **excludes** the 5-byte magic+version; actual byte position from payload start = `5 + var_entry_offset`. |
| `data_offset`     | u32     | Offset from the **first byte after magic and version** to the first byte of the Data region. Stored value **excludes** the 5-byte magic+version; actual byte position from payload start = `5 + data_offset`.     |

The full header is 17 bytes (4 + 1 + 4 + 4 + 4). The header **excluding** magic and version — i.e. the three fields `total_len`, `var_entry_offset`, and `data_offset` — is **12 bytes**.

**Length and offset convention**: All length and offset calculations **exclude** the 4-byte magic and the 1-byte version. The full payload length is always `5 + total_len`. The stored values of `var_entry_offset` and `data_offset` are byte offsets from the first byte of the 12-byte header (i.e. the first byte after magic and version). To obtain the actual byte position from the start of the payload, add 5: VarEntry starts at byte `5 + var_entry_offset`, Data at byte `5 + data_offset`.

Byte order (endianness) for multi-byte integer fields is left to the implementation; it must be consistent for encode and decode.

---

### FixedRegion

- **Position**: Immediately after the header.
- **Content**: Fixed-length values only, written in schema-defined order. No type tags or length prefixes are stored; the layout is determined by the code/schema.
- **Supported fixed types**: `u8`, `u16`, `u32`, `u64`, and fixed-size byte arrays `FixedBytes<N>` (or `[u8; N]`). These are written sequentially into FixedRegion.

---

### VarEntry

- **Position**: Starts at byte `5 + var_entry_offset` from the start of the payload (since `var_entry_offset` is stored relative to the first byte after magic and version).
- **Format**: A sequence of `u32` values. Each value is a **VarEntryOffset**: the offset (from payload start) of the beginning of the corresponding variable-length value in the `Data` region.
- **Count**: The number of VarEntry slots is `(data_offset - var_entry_offset) / 4` (using the stored header values).
- **Length semantics**: The length of the *k*-th variable-length value is inferred as `(VarEntry[k+1] - VarEntry[k])` for *k* &lt; count; the last entry’s length is `(data_offset + total_data_length) - VarEntry[last]`, where `total_data_length` is the data region length: `total_len - data_offset` (because stored `data_offset` excludes magic and version; actual Data start = `5 + data_offset`, so `total_data_length = (5 + total_len) - (5 + data_offset) = total_len - data_offset`).

Implementations may define the exact convention for the last entry (e.g. storing an extra sentinel offset or using `total_len`) as long as encode/decode are consistent.

---

### Data Region

- **Position**: Starts at offset `data_offset` from the start of the payload.
- **Content**: All variable-length values are concatenated in schema-defined order. Each value’s start is recorded in VarEntry; lengths are derived from consecutive VarEntry offsets (or from `total_len` for the last one).

---

### Type Mapping Rules

1. **Fixed-length types in FixedRegion**  
   Types such as `u64` and `FixedBytes<N>` are encoded directly in `[FixedRegion]` in schema order. No VarEntry slots are used for them.

2. **`Vec<u64>`**  
   The elements are written in order into the `data` region as fixed-size values (each 8 bytes). One VarEntry slot is used: it stores the offset of the start of this vector’s segment in `data`. The length (in bytes or in elements) is inferred from VarEntry offsets (or from the end of the data region).

3. **`Bytes` / `String`**  
   The raw bytes (for `String`, typically UTF-8) are appended to the `data` region. One VarEntry slot stores the offset of the start of this value in `data`. Length is inferred from VarEntry offsets.

4. **`Vec<Bytes>`**  
   Treated as *n* separate `Bytes` values: *n* VarEntry slots and *n* contiguous byte ranges in `data`, in order.

5. **`Vec<Vec<u64>>`**  
   Treated as *n* values of type `Vec<u64>`: each inner vector is encoded as in rule 2, with one VarEntry slot per inner vector and corresponding segments in `data`.

6. **Nested (deep) structures**  
   When a type contains nested structures that use this binary layout, each **inner** layer is serialized to bytes using the same layout rules (FixedRegion, VarEntry, Data) but **without** prefixing with `magic` or `version`. Only the **top-level** payload includes the header with `magic` and `version`. The inner bytes are then stored as a variable-length value (e.g. `Bytes`) in the outer layer’s data region. Thus nested structs are represented as raw layout bytes with no magic or version in the middle.

7. **Nesting limit**  
   Deeper nesting of variable-length types (e.g. `Vec<Vec<Bytes>>` or `Vec<Vec<Vec<u64>>>`) is **not** supported at the schema level for unbounded nesting; when nesting is used, each layer is serialized as in rule 6 (inner bytes without magic/version). At most two levels of variable-length structure are allowed (e.g. a top-level variable-length container whose elements are either fixed-length or one level of variable-length).

---

### Invariants

- `var_entry_offset` ≥ size of Header.
- `data_offset` ≥ `var_entry_offset`; typically `data_offset - var_entry_offset` is a multiple of 4 (VarEntry is u32-aligned).
- `total_len` is the number of bytes from the first byte of the `total_len` field to the end of the payload; it **excludes** magic (4) and version (1). The full payload length is therefore `5 + total_len`. The data region length is `(5 + total_len) - data_offset`. Hence `total_len` ≥ `data_offset - 5` in practice (so that the data region is non-negative).
- The number of VarEntry slots must match the number of variable-length fields defined by the schema (including expansion of `Vec<Bytes>` and `Vec<Vec<u64>>` into multiple entries).

---

## References

- Schema and concrete types are defined in code; this spec describes only the wire format and layout rules.
- Related: `0001-basic-types.md` for `Bytes`, `FixedBytes`, and primitive type conventions.
