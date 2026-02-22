//! Decode derive expansion helpers.

use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::common::{add_trait_bounds, add_view_lifetime, collect_fields};

/// Expand a `#[derive(Decode)]` into the corresponding implementation.
pub fn expand_decode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let view_ident = format_ident!("{}View", name);
    let fields = collect_fields(input, "Decode")?;

    let decode_generics = add_trait_bounds(
        &input.generics,
        &fields.field_types,
        quote!(::pufu_core::Decode),
    );
    let view_generics = add_view_lifetime(&decode_generics)?;

    let (decode_impl_generics, decode_ty_generics, decode_where_clause) =
        decode_generics.split_for_impl();
    let (view_impl_generics, view_ty_generics, view_where_clause) = view_generics.split_for_impl();

    let decode_fields = fields
        .field_idents
        .iter()
        .zip(fields.field_types.iter())
        .zip(fields.field_flags.iter())
        .map(|((ident, ty), flag)| {
            quote! {
                let #ident = <#ty as ::pufu_core::Decode>::decode_field::<#flag>(&mut nested_decoder)?;
            }
        });

    let view_fields = fields
        .field_idents
        .iter()
        .zip(fields.field_types.iter())
        .zip(fields.field_vis.iter())
        .map(|((ident, ty), vis)| {
            quote! {
                #vis #ident: <#ty as ::pufu_core::Decode>::View<'a>,
            }
        });

    let field_idents = &fields.field_idents;

    let expanded = quote! {
        struct #view_ident #view_impl_generics #view_where_clause {
            #(#view_fields)*
        }

        impl #decode_impl_generics ::pufu_core::Decode for #name #decode_ty_generics #decode_where_clause {
            type View<'a> = #view_ident #view_ty_generics;

            fn decode_field<'a, const IS_LAST_VAR: bool>(
                decoder: &mut ::pufu_core::Decoder<'a>,
            ) -> Result<Self::View<'a>, ::pufu_core::CodecError> {
                let nested_payload = <Vec<u8> as ::pufu_core::Decode>::decode_field::<IS_LAST_VAR>(
                    decoder,
                )?;
                let mut nested_decoder = ::pufu_core::Decoder::new(nested_payload)?;
                #(#decode_fields)*
                Ok(#view_ident {
                    #(#field_idents),*
                })
            }
        }
    };

    Ok(expanded)
}
