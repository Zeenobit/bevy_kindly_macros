use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Attribute, DeriveInput, Error, Meta};

fn parse_components(attr: &Attribute) -> TokenStream {
    if attr.tokens.is_empty() {
        TokenStream::from(quote! { () })
    } else {
        let mut str = attr.tokens.to_string();
        // This handles the trailing ',' needed to crate a bundle tuple from a single component:
        if str != "()" && str.ends_with(")") && !str.contains(',') {
            str.insert(str.len() - 1, ',');
        }
        str.parse().unwrap()
    }
}

fn parse_bundle(attr: &Attribute) -> TokenStream {
    match attr.parse_meta() {
        Ok(meta) => {
            if let Meta::List(meta_list) = meta {
                let mut nested: Vec<_> = meta_list.nested.iter().collect();
                if nested.len() != 1 {
                    Error::new(attr.span(), "expected a single bundle identifier").into_compile_error()
                } else {
                    let bundle = nested.pop().unwrap();
                    bundle.to_token_stream()
                }
            } else {
                Error::new(attr.span(), "expected a bundle identifier").into_compile_error()
            }
        }
        Err(e) => e.into_compile_error(),
    }
}

#[proc_macro_derive(
    EntityKind,
    attributes(default_components, components, default_bundle, bundle)
)]
pub fn derive_entity_kind(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident;

    let default_bundle = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("default_bundle"))
        .map(parse_bundle);

    let default_components = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("default_components"))
        .map(parse_components);

    if default_bundle.is_some() && default_components.is_some() {
        return Error::new(ident.span(), "you may either define default_bundle or default_components; not both")
            .into_compile_error()
            .into();
    }

    let bundle = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("bundle"))
        .map(parse_bundle);

    let components = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("components"))
        .map(parse_components);

    if bundle.is_some() && components.is_some() {
        return Error::new(ident.span(), "you may either define bundle or components; not both")
            .into_compile_error()
            .into();
    }

    let default_components = default_bundle
        .or(default_components)
        .unwrap_or_else(|| TokenStream::from(quote! { () }));

    let components = bundle
        .or(components)
        .unwrap_or_else(|| TokenStream::from(quote! { () }));

    proc_macro::TokenStream::from(quote! {
        impl EntityKind for #ident {
            type DefaultBundle = #default_components;

            type Bundle = #components;

            unsafe fn from_entity_unchecked(entity: Entity) -> Self {
                Self(entity)
            }

            fn entity(&self) -> Entity {
                self.0
            }
        }
    })
}
