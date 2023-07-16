use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::{Fields, FieldsUnnamed, ItemEnum, parse2, TraitBound};
use syn::token::Plus;
use quote::{quote_spanned, ToTokens};
use syn::parse::Parser;
use syn::spanned::Spanned;

pub fn forwarding2(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    type TraitBounds = Punctuated<TraitBound, Plus>;

    let mut output = TokenStream::new();

    let traits = TraitBounds::parse_terminated.parse2(attr)?;
    let mut item = parse2::<ItemEnum>(item)?;
    let item_ident = item.ident.clone();

    for variant in &mut item.variants {
        match variant.fields.clone() {
            Fields::Named(ns) => {
                if ns.named.len() > 1 {
                    return Err(syn::Error::new(variant.span(), "Only one field allowed on each variant"));
                }
            }
            Fields::Unnamed(us) => {
                if us.unnamed.len() > 1 {
                    return Err(syn::Error::new(variant.span(), "Only one field allowed on each variant"));
                }
            }
            Fields::Unit => {
                let var_ident = variant.ident.clone();
                variant.fields = Fields::Unnamed(
                    parse2::<FieldsUnnamed>(quote_spanned! {var_ident.span() => (#var_ident)})?)
            }
        }
    }

    item.clone().to_tokens(&mut output);

    Ok(output)
}
