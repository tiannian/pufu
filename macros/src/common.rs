use proc_macro2::Span;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, GenericArgument, Type};

pub struct FieldSpec<'a> {
    pub field_idents: Vec<&'a syn::Ident>,
    pub field_types: Vec<&'a Type>,
    pub field_vis: Vec<&'a syn::Visibility>,
    pub field_flags: Vec<proc_macro2::TokenStream>,
}

pub fn collect_fields<'a>(input: &'a DeriveInput, label: &str) -> syn::Result<FieldSpec<'a>> {
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

pub fn add_trait_bounds(
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

pub fn add_view_lifetime(generics: &syn::Generics) -> syn::Result<syn::Generics> {
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
