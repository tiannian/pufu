//! pufu-macros - Derive macro for Codec

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for Codec implementation.
#[proc_macro_derive(Codec)]
pub fn derive_codec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Codec for #name #ty_generics #where_clause {
            // TODO: implement Codec trait
        }
    };

    TokenStream::from(expanded)
}
