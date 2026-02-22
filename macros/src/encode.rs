//! Encode derive expansion helpers.

use quote::quote;
use syn::DeriveInput;

use crate::common::{add_trait_bounds, collect_fields};

/// Expand a `#[derive(Encode)]` into the corresponding implementation.
pub fn expand_encode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let fields = collect_fields(input, "Encode")?;

    let encode_generics = add_trait_bounds(
        &input.generics,
        &fields.field_types,
        quote!(::pufu_core::Encode),
    );
    let (encode_impl_generics, encode_ty_generics, encode_where_clause) =
        encode_generics.split_for_impl();

    let encode_fields = fields
        .field_idents
        .iter()
        .zip(fields.field_flags.iter())
        .map(|(ident, flag)| {
            quote! {
                self.#ident.encode_field::<#flag>(&mut nested_encoder);
            }
        });

    let expanded = quote! {
        impl #encode_impl_generics ::pufu_core::Encode for #name #encode_ty_generics #encode_where_clause {
            fn encode_field<const IS_LAST_VAR: bool>(&self, encoder: &mut ::pufu_core::Encoder) {
                let mut nested_encoder = ::pufu_core::Encoder::new(encoder.endian);
                #(#encode_fields)*

                let mut nested_payload = Vec::new();
                nested_encoder.finalize(&mut nested_payload);
                <Vec<u8> as ::pufu_core::Encode>::encode_field::<IS_LAST_VAR>(
                    &nested_payload,
                    encoder,
                );
            }
        }
    };

    Ok(expanded)
}
