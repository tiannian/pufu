# Trait Layout Taxonomy for Binary Serialization

## Overview

This specification describes the trait taxonomy used in the codebase to classify types for the binary serialization protocol. The traits distinguish **fixed-length types**, **first-order variable-length types**, and **second-order variable-length types**. The classification drives layout decisions: fixed-length values go into the Fixed region; variable-length values consume VarEntry slots and bytes in the Data region. The wire format itself is defined in `0010-binary-serde.md`; this document specifies the type-level contracts and how to use them.

## Detailed Specifications

### Summary of the Three Categories

| Category                         | Trait    | Wire behaviour                                      | VarEntry slots per value |
|----------------------------------|----------|-----------------------------------------------------|---------------------------|
| Fixed-length                     | `ZcFixed`| Stored in FixedRegion in schema order               | 0                         |
| First-order variable-length      | `ZcVar1` | One contiguous segment in Data; one offset in VarEntry | 1                      |
| Second-order variable-length     | `ZcVar2` | Multiple segments in Data; one VarEntry slot per inner value | N (number of inner values) |

---

### Fixed-Length Types: `ZcFixed`

**Purpose**: Mark types whose serialized size is known at compile time. These are stored in the **FixedRegion** in schema-defined order. No VarEntry slots are used.

**Trait**:

- `ZcFixed`: requires two associated constants:
  - `SIZE: usize` — serialized size in bytes.
  - `ALIGN: usize` — alignment requirement (e.g. for correct placement in the fixed region).

**Typical implementations**:

- Primitives: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, and the corresponding signed and `bool` types.
- Fixed-size arrays: `[T; N]` where `T: ZcFixed`; `SIZE = T::SIZE * N`, `ALIGN = T::ALIGN`.

**Usage**: Any field type that implements `ZcFixed` is laid out in the Fixed region. The derive macro (or manual layout logic) places them in order; no offsets are stored for them in VarEntry.

---

### First-Order Variable-Length Types: `ZcVar1`

**Purpose**: Mark types that represent a **single** variable-length segment. Each value consumes **exactly one** VarEntry slot; the segment’s length is inferred from the next VarEntry offset (or the end of the data region).

**Trait**:

- `ZcVar1`: requires one associated constant:
  - `ELEM_SIZE: Option<usize>`:
    - `None`: **bytes-like** segment (arbitrary byte length). Used for `Vec<u8>`, `String`, `&str`, `&[u8]`, etc.
    - `Some(sz)`: **fixed-element** segment; the segment is a sequence of fixed-size elements, each `sz` bytes. Used for `Vec<T>` where `T: ZcFixed` and `T` is not `u8` (e.g. `Vec<u64>`, `Vec<[u8; 32]>`). Element count = segment_byte_length / sz.

**Typical implementations**:

- Bytes-like: `Vec<u8>`, `String`, `&str`, `&[u8]`, `&mut [u8]`.
- Fixed-element vectors: `Vec<T>` and slice views `&[T]`, `&mut [T]` for `T: ZcFixed` and `T` not equal to `u8` (e.g. `Vec<u64>`, `Vec<[u8; 32]>`). Disambiguation between `Vec<u8>` (bytes-like) and fixed-elem `Vec<T>` is done via a sealed marker so that `Vec<u8>` always gets `ELEM_SIZE = None`.

**Usage**: Any field type that implements `ZcVar1` is treated as one variable-length value: one VarEntry slot stores the offset of its segment in the Data region; the segment is written contiguously there. Length is derived from adjacent VarEntry offsets (or from the data region length for the last entry).

---

### Second-Order Variable-Length Types: `ZcVar2`

**Purpose**: Mark types that represent a **vector of** variable-length segments (each segment is first-order). Each **inner** value consumes **one** VarEntry slot; the outer type therefore consumes **N** VarEntry slots, where N is the number of inner values (e.g. the length of the outer `Vec`).

**Trait**:

- `ZcVar2`: requires an associated type:
  - `type Inner: ZcVar1` — the type of each element; each element is encoded as a ZcVar1 segment (one VarEntry slot and one contiguous range in Data).

**Typical implementations**:

- `Vec<Vec<u8>>`: inner type `Vec<u8>` is ZcVar1 (bytes-like). Each inner vector gets one VarEntry slot and one byte range in Data.
- `Vec<Vec<u64>>`: inner type `Vec<u64>` is ZcVar1 (fixed-element). Each inner vector gets one VarEntry slot and one segment of u64s in Data.
- `Vec<String>`: inner type `String` is ZcVar1 (bytes-like). Each string gets one VarEntry slot and one byte range in Data.

**Usage**: A field of type implementing `ZcVar2` expands to N variable-length values in schema order: N VarEntry slots and N contiguous segments in the Data region. The derive macro may enforce that at most one ZcVar2 field exists and that it is the final variable-length group, so that the number of VarEntry slots is well-defined and matches the layout.

---

### How Type Choice Distinguishes the Three Kinds

- **Fixed-length**: Use a type that implements **only** `ZcFixed` (e.g. `u64`, `[u8; 32]`). No VarEntry; stored in FixedRegion.
- **First-order variable-length**: Use a type that implements **ZcVar1** (e.g. `Vec<u8>`, `String`, `Vec<u64>`). One VarEntry slot; one segment in Data.
- **Second-order variable-length**: Use a type that implements **ZcVar2** with `Inner: ZcVar1` (e.g. `Vec<Vec<u8>>`, `Vec<Vec<u64>>`). N VarEntry slots; N segments in Data.

The traits are mutually exclusive in the sense of layout role: a type is used either as fixed (ZcFixed), or as one var segment (ZcVar1), or as many var segments (ZcVar2). The same concrete type (e.g. `Vec<u8>`) does not implement ZcFixed or ZcVar2; it implements only ZcVar1.

---

### Relation to the Wire Format

- **FixedRegion** (see `0010-binary-serde.md`): contains only values of types implementing `ZcFixed`, in schema order.
- **VarEntry**: number of slots = sum over fields of (1 for each ZcVar1 field) + (N for each ZcVar2 field, where N is the length of that field’s value). Order of slots follows schema order and, for ZcVar2, the order of inner elements.
- **Data region**: concatenation of all variable-length segments in the same order as VarEntry offsets; each segment corresponds to one ZcVar1 value or one element of a ZcVar2 value.

---

## References

- `0010-binary-serde.md` — wire layout (Header, FixedRegion, VarEntry, Data) and type mapping rules.
- `core/src/zc/` — trait definitions (`traits`), implementations (`fixed`, `var1`), and sealing/disambiguation (`sealed`).
