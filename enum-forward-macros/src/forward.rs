// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{GenericParam, Generics, ItemEnum, parse2, TypeParam};

use crate::common::{variant_patterns, VariantInfo};
use crate::error::Result;

pub fn forwarding2(item: TokenStream) -> Result<TokenStream> {
    let mut output = TokenStream::new();

    let item = parse2::<ItemEnum>(item)?;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let item_ident = item.ident.clone();

    let mut impl_generics = parse2::<Generics>(impl_generics.to_token_stream())?;

    let input_ty = Ident::new("I", Span::call_site());
    let output_ty = Ident::new("R", Span::call_site());

    impl_generics.params.push(GenericParam::Type(TypeParam::from(input_ty.clone())));
    impl_generics.params.push(GenericParam::Type(TypeParam::from(output_ty.clone())));

    let types = variant_patterns(&item).map(|v| Ok((v?.inner_ty).clone())).collect::<Result<Vec<_>>>()?;
    let additional_wheres = types.iter().unique().map(|ty| {
        Ok(quote!(#ty : enum_forward::Forward<#input_ty, Output=#output_ty>))
    }).collect::<Result<Vec<_>>>()?;
    let where_clause = match where_clause {
        None => { quote!(where #(#additional_wheres),*) }
        Some(w) => { quote!(#w, #(#additional_wheres),*) }
    };

    let arms = variant_patterns(&item).map(|v| {
        let VariantInfo { pattern, .. } = v?;
        Ok(quote!(#pattern => {enum_forward::Forward::forward(value, input)}))
    }).collect::<Result<Vec<_>>>()?;

    output.extend(quote! {
        impl<#input_ty,#output_ty> enum_forward::Forward<#input_ty> for #item_ident #ty_generics #where_clause {
            type Output = #output_ty;

            fn forward(&self, input : &#input_ty) -> #output_ty {
                return match self {
                    #(#arms),*
                }
            }
        }
    });

    Ok(output)
}

