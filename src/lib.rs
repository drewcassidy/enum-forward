use crate::derive::{derive_enum_from2, derive_enum_tryinto2};

mod derive;
mod common;
mod error;
mod forward;

#[proc_macro_derive(From)]
pub fn derive_enum_from(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive_enum_from2(item.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error(),
    }
}

#[proc_macro_derive(TryInto)]
pub fn derive_enum_tryinto(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive_enum_tryinto2(item.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error(),
    }
}


#[proc_macro_attribute]
pub fn forwarding(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match forward::forwarding2(attr.into(), item.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}