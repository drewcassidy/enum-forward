// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use proc_macro2::{Span};

#[derive(Clone)]
pub enum Error {
    MultipleMembers(Span),
    UnitVariant(Span),
    DuplicateType(Span),
    Other(Span, String),
    Syn(syn::Error),
}

impl Into<syn::Error> for Error {
    fn into(self) -> syn::Error {
        match self {
            Error::MultipleMembers(span) => {
                syn::Error::new(span,
                                "Enum variant has multiple members, and cannot be converted to or from an inner type")
            }
            Error::UnitVariant(span) => {
                syn::Error::new(span,
                                "Enum variant is a unit variant, and cannot be converted to or from an inner type")
            }
            Error::DuplicateType(span) => {
                syn::Error::new(span, "Enum has multiple variants with the same type.")
            }
            Error::Other(span, msg) => {
                syn::Error::new(span, msg)
            }
            Error::Syn(err) => err,
        }
    }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Self::Syn(value)
    }
}

impl Error {
    pub fn to_compile_error(self) -> proc_macro::TokenStream {
        <Self as Into<syn::Error>>::into(self).to_compile_error().into()
    }
}

pub type Result<T> = std::result::Result<T, Error>;