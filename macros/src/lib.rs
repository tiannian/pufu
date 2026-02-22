//! pufu-macros - Derive macros for Encode and Decode

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod common;
mod decode;
mod encode;

#[proc_macro_derive(Encode)]
/// Derive `pufu_core::Encode` for named-field structs.
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match encode::expand_encode(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Decode)]
/// Derive `pufu_core::Decode` for named-field structs.
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match decode::expand_decode(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };

    TokenStream::from(expanded)
}
