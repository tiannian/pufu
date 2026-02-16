//! Trait definitions for the Zc layout taxonomy.

/// Fixed-size types stored in the Fixed region.
pub trait ZcFixed {
    const SIZE: usize;
    const ALIGN: usize;
}

/// One variable-length segment (1st level).
/// The derive macro can treat any field type implementing this trait as "consumes 1 var_idx entry".
pub trait ZcVar1 {
    /// None => bytes-like segment
    /// Some(sz) => segment is a sequence of fixed-size elements, each `sz` bytes
    const ELEM_SIZE: Option<usize>;
}

/// Second-level variable array: Vec<Inner> where Inner is a "var1 segment".
/// The derive macro can enforce: at most one ZcVar2 field, and it must be the final var group.
pub trait ZcVar2 {
    type Inner: ZcVar1;
}
