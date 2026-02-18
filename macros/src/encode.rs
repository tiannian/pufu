use quote::quote;
use syn::DeriveInput;

use crate::common::{add_trait_bounds, collect_fields};

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
                self.#ident.encode_field::<#flag>(encoder);
            }
        });

    let expanded = quote! {
        impl #encode_impl_generics ::pufu_core::Encode for #name #encode_ty_generics #encode_where_clause {
            fn encode_field<const IS_LAST_VAR: bool>(&self, encoder: &mut ::pufu_core::Encoder) {
                let _ = IS_LAST_VAR;
                #(#encode_fields)*
            }
        }
    };

    Ok(expanded)
}
