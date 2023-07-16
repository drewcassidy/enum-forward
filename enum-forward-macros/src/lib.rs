// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::convert::{derive_enum_from2, derive_enum_tryinto2};

mod convert;
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