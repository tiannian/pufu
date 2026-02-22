//! Config for binary serialization protocol (see specs/0017-config.md).

use crate::Endian;

/// Default magic bytes (e.g. b"svsd").
pub const DEFAULT_MAGIC: [u8; 4] = [0x73, 0x76, 0x73, 0x64];

/// Config carrying magic, version, and endianness for encode/decode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Four-byte magic identifier at the start of the payload (serialized).
    pub magic: [u8; 4],
    /// Protocol version byte (serialized).
    pub version: u8,
    /// Byte order for multi-byte integer fields. Not serialized; used only at encode/decode time.
    pub endian: Endian,
}

impl Config {
    /// Returns a new builder with default values (magic, version 1, little-endian).
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            magic: DEFAULT_MAGIC,
            version: 1,
            endian: Endian::Little,
        }
    }
}

/// Builder for Config.
#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    magic: Option<[u8; 4]>,
    version: Option<u8>,
    endian: Option<Endian>,
}

impl ConfigBuilder {
    /// Sets the magic bytes.
    pub fn magic(mut self, magic: [u8; 4]) -> Self {
        self.magic = Some(magic);
        self
    }

    /// Sets the protocol version.
    pub fn version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the endianness.
    pub fn endian(mut self, endian: Endian) -> Self {
        self.endian = Some(endian);
        self
    }

    /// Sets endianness to big-endian.
    pub fn big(self) -> Self {
        self.endian(Endian::Big)
    }

    /// Sets endianness to little-endian.
    pub fn little(self) -> Self {
        self.endian(Endian::Little)
    }

    /// Sets endianness to the platform's native byte order.
    pub fn native(self) -> Self {
        self.endian(Endian::Native)
    }

    /// Builds a Config; missing fields use defaults (DEFAULT_MAGIC, version 1, Little).
    pub fn build(self) -> Config {
        Config {
            magic: self.magic.unwrap_or(DEFAULT_MAGIC),
            version: self.version.unwrap_or(1),
            endian: self.endian.unwrap_or(Endian::Little),
        }
    }
}
