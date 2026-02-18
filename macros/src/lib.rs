//! pufu-macros - Derive macros for Encode and Decode

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, GenericArgument, Type};

#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match expand_encode(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Decode)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let expanded = match expand_decode(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };

    TokenStream::from(expanded)
}

fn expand_encode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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

fn expand_decode(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
                let #ident = <#ty as ::pufu_core::Decode>::decode_field::<#flag>(decoder)?;
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
                let _ = IS_LAST_VAR;
                #(#decode_fields)*
                Ok(#view_ident {
                    #(#field_idents),*
                })
            }
        }
    };

    Ok(expanded)
}

struct FieldSpec<'a> {
    field_idents: Vec<&'a syn::Ident>,
    field_types: Vec<&'a Type>,
    field_vis: Vec<&'a syn::Visibility>,
    field_flags: Vec<proc_macro2::TokenStream>,
}

fn collect_fields<'a>(input: &'a DeriveInput, label: &str) -> syn::Result<FieldSpec<'a>> {
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
            syn::Fields::Unnamed(fields) => {
                return Err(syn::Error::new(
                    fields.span(),
                    format!("{label} can only be derived for named-field structs"),
                ));
            }
            syn::Fields::Unit => {
                return Err(syn::Error::new(
                    data.struct_token.span(),
                    format!("{label} cannot be derived for unit structs"),
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                input.span(),
                format!("{label} can only be derived for structs"),
            ));
        }
    };

    let mut field_idents = Vec::with_capacity(fields.len());
    let mut field_types = Vec::with_capacity(fields.len());
    let mut field_vis = Vec::with_capacity(fields.len());

    for field in &fields {
        let ident = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new(
                field.span(),
                format!("{label} can only be derived for named fields"),
            )
        })?;
        field_idents.push(ident);
        field_types.push(&field.ty);
        field_vis.push(&field.vis);
    }

    let mut var_field_indices = Vec::new();
    let mut var2_indices = Vec::new();

    for (idx, ty) in field_types.iter().enumerate() {
        match field_var_kind(ty) {
            VarKind::Var1 => var_field_indices.push(idx),
            VarKind::Var2 => {
                var_field_indices.push(idx);
                var2_indices.push(idx);
            }
            VarKind::None => {}
        }
    }

    let last_var_index = if var_field_indices.is_empty() {
        if field_types.is_empty() {
            None
        } else {
            Some(field_types.len() - 1)
        }
    } else {
        Some(*var_field_indices.last().expect("non-empty"))
    };

    if let Some(last_var_index) = last_var_index {
        for idx in var2_indices {
            if idx != last_var_index {
                return Err(syn::Error::new(
                    field_types[idx].span(),
                    "var2 data must be encoded/decoded as the last variable field",
                ));
            }
        }
    }

    let field_flags = field_types
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            if Some(idx) == last_var_index {
                quote!(true)
            } else {
                quote!(false)
            }
        })
        .collect::<Vec<_>>();

    Ok(FieldSpec {
        field_idents,
        field_types,
        field_vis,
        field_flags,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VarKind {
    None,
    Var1,
    Var2,
}

fn field_var_kind(ty: &Type) -> VarKind {
    let inner = match vec_inner_type(ty) {
        Some(inner) => inner,
        None => return VarKind::None,
    };

    if vec_inner_type(inner).is_some() {
        VarKind::Var2
    } else {
        VarKind::Var1
    }
}

fn vec_inner_type(ty: &Type) -> Option<&Type> {
    let type_path = match ty {
        Type::Path(type_path) => type_path,
        _ => return None,
    };

    if type_path.qself.is_some() {
        return None;
    }

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Vec" {
        return None;
    }

    let args = match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args,
        _ => return None,
    };

    if args.args.len() != 1 {
        return None;
    }

    match args.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

fn add_trait_bounds(
    generics: &syn::Generics,
    field_types: &[&Type],
    trait_path: proc_macro2::TokenStream,
) -> syn::Generics {
    let mut generics = generics.clone();
    let where_clause = generics.make_where_clause();
    for ty in field_types {
        where_clause
            .predicates
            .push(syn::parse_quote!(#ty: #trait_path));
    }
    generics
}

fn add_view_lifetime(generics: &syn::Generics) -> syn::Result<syn::Generics> {
    let mut generics = generics.clone();
    let has_a = generics.lifetimes().any(|lt| lt.lifetime.ident == "a");
    if !has_a {
        generics.params.insert(
            0,
            syn::GenericParam::Lifetime(syn::LifetimeParam::new(syn::Lifetime::new(
                "'a",
                Span::call_site(),
            ))),
        );
    }
    Ok(generics)
}
