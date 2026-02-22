# Config for Binary Serialization Protocol

## Overview

This specification defines the **Config** type used to carry magic bytes, protocol version, and endianness for the binary serialization protocol. Config is built via **ConfigBuilder** and is used to initialize the **Encoder** (see `0011-encoder.md`) and **Decoder** (see `0012-decoder.md`), ensuring consistent magic, version, and byte order for encode and decode.

## Detailed Specifications

### Config Structure

| Field     | Type        | Description                                                                 |
|-----------|-------------|-----------------------------------------------------------------------------|
| `magic`   | `[u8; 4]`   | Four-byte magic identifier at the start of the payload (e.g. `b"svsd"`).   |
| `version` | `u8`        | Protocol version byte.                                                     |
| `endian`  | `Endian`   | Byte order for multi-byte integer fields. **Not serialized**: used only at encode/decode time; never written to or read from the wire. |

Invariants: none beyond type constraints. Encoder and Decoder use the same config so that written and read byte order and header prefix match.

---

### ConfigBuilder

ConfigBuilder constructs a **Config** with optional overrides. Defaults (e.g. magic `b"svsd"`, version `1`, little-endian) are implementation-defined; the builder allows overriding each field.

| Method / field | Description |
|----------------|-------------|
| `magic(magic: [u8; 4]) -> Self` | Sets the magic bytes. |
| `version(version: u8) -> Self`   | Sets the protocol version. |
| `endian(endian: Endian) -> Self`| Sets the endianness. |
| `big() -> Self`                  | Sets endianness to big-endian. |
| `little() -> Self`               | Sets endianness to little-endian. |
| `native() -> Self`               | Sets endianness to the platform’s native byte order. |
| `build(self) -> Config`         | Produces a `Config` from the current builder state. |

Chaining is supported (each setter returns `Self`). Missing fields use the implementation’s defaults when `build()` is called.

---

### Usage

- **Encoder**: Encoder is constructed with a `Config` (or equivalent). The config supplies magic, version, and endianness for methods that write the full header (e.g. `finalize_with_magic_version`). See `0011-encoder.md`.
- **Decoder**: Decoder is constructed from a buffer and a `Config`. The config supplies expected magic, version, and endianness for validation and for accessors `magic()`, `version()`, and `endian()`. See `0012-decoder.md`.

---

## References

- Wire layout and header: `0010-binary-serde.md`.
- Encoder: `0011-encoder.md`.
- Decoder: `0012-decoder.md`.
