// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::quote;
use syn::{Fields, ItemEnum, Type, Variant};
use syn::spanned::Spanned;
use crate::error::{Error, Result};

pub(crate) struct VariantInfo<'a> {
    pub variant: &'a Variant,
    pub inner_ty: &'a Type,
    pub pattern: TokenStream,
}

pub(crate) fn variant_patterns(item: &ItemEnum) -> impl Iterator<Item=Result<VariantInfo>> {
    let item_ident = item.ident.clone();

    item.variants.iter().map(move |variant| {
        let var_ident = &variant.ident;
        return match &variant.fields {
            Fields::Named(ns) => {
                if ns.named.len() < 1 {
                    return Err(Error::UnitVariant(variant.span()));
                }
                if ns.named.len() > 1 {
                    return Err(Error::MultipleMembers(variant.span()));
                }
                let field_ident = &ns.named[0].ident.clone().unwrap();
                let inner_ty = &ns.named[0].ty;
                let pattern = quote! { #item_ident::#var_ident{#field_ident : value} };
                Ok(VariantInfo { variant, inner_ty, pattern })
            }
            Fields::Unnamed(us) => {
                if us.unnamed.len() < 1 {
                    return Err(Error::UnitVariant(variant.span()));
                }
                if us.unnamed.len() > 1 {
                    return Err(Error::MultipleMembers(variant.span()));
                }
                let inner_ty = &us.unnamed[0].ty;
                let pattern = quote! { #item_ident::#var_ident(value) };
                Ok(VariantInfo { variant, inner_ty, pattern })
            }
            Fields::Unit => {
                Err(Error::UnitVariant(variant.span()))
            }
        };
    })
}

pub(crate) fn replace_ident(ts: TokenStream, from: Ident, to: Ident) -> TokenStream {
    ts.into_iter().map(
        |tt| {
            match tt {
                TokenTree::Group(g) => {
                    TokenTree::Group(Group::new(g.delimiter(), replace_ident(g.stream(), from.clone(), to.clone())))
                }
                TokenTree::Ident(i) if i.to_string() == to.to_string() => {
                    TokenTree::Ident(to.clone())
                }
                other => other
            }
        }
    ).collect()
}