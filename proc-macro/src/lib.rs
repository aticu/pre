//! This crate contains the implementation for attributes used in the `pre` crate.
//!
//! Refer to the documentation of the `pre` crate for more information.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{emit_warning, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, Item, ItemFn};

use crate::{
    assert_pre::AssertPreVisitor,
    def::{DefPreAttr, DefPreModule},
    precondition::{Precondition, PreconditionList},
};

mod assert_pre;
mod def;
mod precondition;

cfg_if::cfg_if! {
    if #[cfg(feature = "const-generics-impl")] {
        mod const_generics_impl;
        pub(crate) use crate::const_generics_impl::{render_assert_pre, render_pre};
    } else if #[cfg(feature = "struct-impl")] {
        mod struct_impl;
        pub(crate) use crate::struct_impl::{render_assert_pre, render_pre};
    } else {
        compile_error!("you must choose one of the features providing an implementation")
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn pre(attr: TokenStream, function: TokenStream) -> TokenStream {
    let dummy_function: TokenStream2 = function.clone().into();
    proc_macro_error::set_dummy(quote! {
        #dummy_function
    });

    let preconditions = parse_macro_input!(attr as PreconditionList<Precondition>);
    let function = parse_macro_input!(function as ItemFn);

    let output = render_pre(preconditions, function);

    // Reset the dummy here, in case errors were emitted in `render_pre`.
    // This will use the most up-to-date version of the generated code.
    proc_macro_error::set_dummy(quote! {
        #dummy_function
    });

    output.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn check_pre(attr: TokenStream, item: TokenStream) -> TokenStream {
    let dummy_item: TokenStream2 = item.clone().into();
    proc_macro_error::set_dummy(quote! {
        #dummy_item
    });

    if !attr.is_empty() {
        let attr: TokenStream2 = attr.into();
        emit_warning!(attr, "this does not do anything and is ignored");
    }

    let mut item = parse_macro_input!(item as Item);

    AssertPreVisitor.visit_item_mut(&mut item);

    let output = quote! {
        #item
    };

    // Reset the dummy here, in case errors were emitted in visiting the syntax tree.
    // This will use the most up-to-date version of the generated code.
    proc_macro_error::set_dummy(quote! {
        #output
    });

    output.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn def_pre(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as DefPreAttr);
    let item = parse_macro_input!(item as DefPreModule);

    let output = item.render(attr);

    output.into()
}
