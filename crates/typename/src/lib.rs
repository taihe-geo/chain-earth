extern crate proc_macro;

use std::any::type_name;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Type};

#[proc_macro_derive(TypeName)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    quote! (
        impl TypeName for #ident{}
    )
    .into()
}
