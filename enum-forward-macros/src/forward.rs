// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Attribute, FnArg, GenericParam, Generics, ItemEnum, Lifetime, LifetimeParam, parse2, Pat, PatIdent, Signature, Token, TraitBound, Type, TypeParam, TypeParamBound, TypePath, Visibility, WherePredicate, PredicateType, WhereClause, ReturnType, TypeTuple};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Brace;


use crate::common::{variant_patterns, VariantInfo, lifetimeify};
use crate::error::{Error, Result};

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

struct InputFn {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub sig: Signature,
    pub body: Option<TokenStream>,
}

impl Parse for InputFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let sig: Signature = input.parse()?;

        let body = if input.peek(Brace) {
            Some(input.parse::<TokenStream>()?)
        } else {
            input.parse::<Token!(;)>()?;
            None
        };

        Ok(InputFn { attrs, vis, sig, body })
    }
}

struct InputAttr {
    pub pat: Option<Pat>,
    pub ty: Type,
    pub as_token: Token!(as),
    pub traits: Punctuated<TraitBound, Token!(+)>,
}

impl Parse for InputAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = if input.peek2(Token!(:)) {
            let pat: Pat = Pat::parse_single(input)?;
            let _: Token!(:) = input.parse()?;
            Some(pat)
        } else {
            None
        };

        let ty: Type = input.parse()?;
        let as_token: Token!(as) = input.parse()?;
        let traits = Punctuated::<TraitBound, Token!(+)>::parse_terminated(input)?;

        Ok(InputAttr { pat, ty, as_token, traits })
    }
}

pub fn forward_to(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attr: InputAttr = parse2(attr)?;
    let item: InputFn = parse2(item)?;
    let item_attrs = item.attrs;
    let item_vis = item.vis;
    let item_sig = item.sig;

    let result_ty = match item_sig.output.clone() {
        ReturnType::Default => { Type::Tuple(TypeTuple { paren_token: Default::default(), elems: Default::default() }) }
        ReturnType::Type(_, bt) => { *bt.clone() }
    };

    let ident = item_sig.clone().ident;

    let mut inner = TokenStream::new();

    let mut receiver: Option<(Type, Pat)> = None;
    let mut args: Vec<(Type, Pat)> = vec![];

    let caller_ty: Type = parse2(quote!(Caller))?;

    let self_: PatIdent = PatIdent {
        attrs: vec![],
        by_ref: None,
        mutability: None,
        ident: Ident::new("_self", Span::call_site()),
        subpat: None,
    };

    for input in item_sig.clone().inputs.iter().rev() {
        match input {
            FnArg::Typed(typed) => {
                let pat = *typed.pat.clone();

                match (Some(pat.clone()), *typed.ty.clone()) {
                    (p, t) if p == attr.pat.clone() && t == attr.ty.clone() => {
                        receiver = Some((t.clone(), pat.clone()))
                    }
                    (p, t) if p == attr.pat.clone() => {
                        return Err(Error::MismatchedArgType(t.span()));
                    }
                    (p, t) if t == attr.ty.clone() => {
                        return Err(Error::MismatchedArgType(p.span()));
                    }
                    (p, t) => {
                        args.insert(0, (t.clone(), pat.clone()))
                    }
                }
            }
            FnArg::Receiver(rec) => {
                if receiver.is_none() {
                    receiver = Some((attr.ty.clone(), Pat::Ident(self_.clone())));
                } else {
                    args.insert(0, (*rec.ty.clone(), Pat::Ident(self_.clone())))
                }
            }
        }
    }

    let blanket_lt = Lifetime::new("'_blanket", Span::call_site());
    let mut lifetimes = HashSet::<Lifetime>::from([blanket_lt.clone()]);

    for (ref mut ty, _) in &mut args {
        (*ty) = lifetimeify(ty.clone(), &blanket_lt, &mut lifetimes);
    }

    let mut struct_generics = item_sig.clone().generics;
    struct_generics.params.extend([blanket_lt.clone()].iter().map(|lt| GenericParam::Lifetime(LifetimeParam {
        attrs: vec![],
        lifetime: lt.clone(),
        colon_token: None,
        bounds: Default::default(),
    })));

    let (impl_generics, ty_generics, where_clause) = struct_generics.split_for_impl();
    let struct_ident = format_ident!("{}Visitor", ident.clone());
    let struct_items = &args.iter().map(|(ty, pat)| {
        quote!(#pat : #ty)
    }).collect_vec();

    inner.extend(
        quote! {
            struct #struct_ident #ty_generics #where_clause {
                #(#struct_items),*,
                _phantom : std::marker::PhantomData<&#blanket_lt i32>
            }

        }
    );

    inner.extend(impl_forward_variants(struct_generics.clone(), Type::Verbatim(struct_ident.clone().into_token_stream()), result_ty.clone(), attr.traits.clone()));


    if item.body.is_some() {
        unimplemented!()
    }

    let output = quote! {
        #(#item_attrs)*
        #item_vis #item_sig {
            #inner
        }
    };


    Ok(output)
}

fn impl_forward_variants(mut generics: Generics, input_ty: Type, result_ty: Type, traits_bounds: Punctuated<TraitBound, Token!(+)>)
                         -> Result<TokenStream> {

    let input_generics = generics.clone();
    let (_, input_ty_generics, _) = input_generics.split_for_impl();


    let blanket_ty = Ident::new("B", Span::call_site());
    generics.params.extend([GenericParam::Type(TypeParam { attrs: vec![], ident: blanket_ty.clone(), colon_token: None, bounds: Default::default(), eq_token: None, default: None })].into_iter());
    let mut where_clause = generics.where_clause.unwrap_or(WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause.predicates.extend(
        [WherePredicate::Type(
            PredicateType {
                lifetimes: None,
                bounded_ty: Type::Verbatim(blanket_ty.to_token_stream()),
                colon_token: Default::default(),
                bounds: traits_bounds.iter().map(
                    |tb| { TypeParamBound::Trait(tb.clone()) }
                ).collect(),
            }
        )].into_iter()
    );
    generics.where_clause = Some(where_clause);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();



    return Ok(quote! {
        impl #impl_generics enum_forward::Forward<#input_ty::<'a, '_blanket>> for #blanket_ty #where_clause {

        }
    });
}