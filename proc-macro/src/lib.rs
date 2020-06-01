use proc_macro::TokenStream;
use syn::{parse_macro_input, ExprCall, ItemFn};

use crate::precondition::{Precondition, PreconditionHolds};

mod precondition;

#[cfg(feature = "struct-impl")]
mod struct_impl;
#[cfg(feature = "struct-impl")]
use struct_impl::{render_assert_precondition, render_pre};

#[cfg(feature = "const-generics-impl")]
mod const_generics_impl;
#[cfg(feature = "const-generics-impl")]
use const_generics_impl::{render_assert_precondition, render_pre};

#[proc_macro_attribute]
pub fn pre(attr: TokenStream, function: TokenStream) -> TokenStream {
    let preconditions = parse_macro_input!(attr as Precondition);
    let function = parse_macro_input!(function as ItemFn);

    let output = render_pre(preconditions, function);

    output.into()
}

#[proc_macro_attribute]
pub fn assert_precondition(attr: TokenStream, call: TokenStream) -> TokenStream {
    let preconditions = parse_macro_input!(attr as PreconditionHolds);
    let call = parse_macro_input!(call as ExprCall);

    let output = render_assert_precondition(preconditions, call);

    output.into()
}
