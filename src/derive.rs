// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{ItemEnum, parse2};
use syn::spanned::Spanned;

use crate::common::{variant_patterns, VariantInfo};
use crate::error::{Error, Result};

pub(crate) fn derive_enum_from2(item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<ItemEnum>(item)?;
    let item_ident = item.ident.clone();
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut output = TokenStream::new();

    let mut tys = HashSet::<String>::new();

    for v in variant_patterns(&item) {
        let VariantInfo { inner_ty, pattern, .. } = v.map_err(
            |e| match e {
                Error::UnitVariant(s) => {
                    // provide more info for unit variants in this case
                    Error::Other(s, "Cannot use unit enum variants with `Derive(From)`. \
                    If you are using the `forwarder` macro, \
                    make sure it is before any derives.".into())
                }
                _ => e.clone()
            }
        )?;

        // check for duplicate types. This will fail anyways due to duplicate From<T>
        // impls, but this error should be more readable
        let inner_ty_name = inner_ty.to_token_stream().to_string();
        if !tys.contains(&inner_ty_name) {
            tys.insert(inner_ty_name);
        } else {
            return Err(Error::DuplicateType(inner_ty.span()));
        }

        output.extend(quote! {
            impl #impl_generics ::core::convert::From<#inner_ty> for #item_ident #ty_generics #where_clause {
                fn from(value : #inner_ty) -> #item_ident {
                    #pattern
                }
            }
        });
    }

    Ok(output)
}

pub(crate) fn derive_enum_tryinto2(item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<ItemEnum>(item)?;
    let item_ident = item.ident.clone();
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let mut output = TokenStream::new();

    let mut tys = HashSet::<String>::new();

    let err_map = |e: Error| match e {
        Error::UnitVariant(s) => {
            // provide more info for unit variants in this case
            Error::Other(s, "Cannot use unit enum variants with `Derive(TryInto)`. \
                    If you are using the `forwarder` macro, \
                    make sure it is before any derives.".into())
        }
        _ => e.clone()
    };

    for v in variant_patterns(&item) {
        let VariantInfo { inner_ty: try_ty, .. } = v.map_err(err_map)?;

        let try_ty_name = try_ty.to_token_stream().to_string();
        // check if this type has already been implemented
        if !tys.contains(&try_ty_name) {
            tys.insert(try_ty_name.clone());

            let arms = variant_patterns(&item).map(
                |v| -> Result<TokenStream> {
                    let VariantInfo { variant, inner_ty, pattern, .. } = v.map_err(err_map)?;

                    if inner_ty.to_token_stream().to_string() == try_ty.to_token_stream().to_string() {
                        return Ok(quote!(#pattern => Ok(value)));
                    } else {
                        let msg = format!("Cannot convert {}::{} to {}", item_ident, variant.ident, try_ty_name);
                        return Ok(quote!(#pattern => Err(#msg)));
                    }
                }
            ).collect::<Result::<Vec::<_>>>()?;

            output.extend(quote! {
            impl #impl_generics ::core::convert::TryInto<#try_ty> for #item_ident #ty_generics #where_clause {
                type Error  = &'static str;

                fn try_into(self) -> ::core::result::Result<#try_ty, <Self as core::convert::TryInto<#try_ty>>::Error> {
                    match self {
                        #(#arms),*
                    }
                }
            }
        })
        }
    }

    Ok(output)
}