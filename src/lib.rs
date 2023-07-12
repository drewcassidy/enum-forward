use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::{Fields, FieldsUnnamed, ItemEnum, parse2, TraitBound};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Plus;

#[proc_macro_attribute]
pub fn forwarder(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match forwarder2(attr.into(), item.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn forwarder2(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    type TraitBounds = Punctuated<TraitBound, Plus>;

    let traits = TraitBounds::parse_terminated.parse2(attr)?;
    let mut item = parse2::<ItemEnum>(item)?;

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

    Ok(item.to_token_stream())
}