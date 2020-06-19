//! This crate contains the implementation for attributes used in the `pre` crate.
//!
//! Refer to the documentation of the `pre` crate for more information.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, File};

use crate::{
    pre_attr::PreAttrVisitor,
    pre_defs_for::{DefinitionsForAttr, DefinitionsForModule},
};

mod assert_pre;
mod call;
mod helpers;
mod pre_attr;
mod pre_defs_for;
mod precondition;

cfg_if::cfg_if! {
    if #[cfg(nightly)] {
        mod const_generics_impl;
        pub(crate) use crate::const_generics_impl::{render_assert_pre, render_pre};
    } else {
        mod struct_impl;
        pub(crate) use crate::struct_impl::{render_assert_pre, render_pre};
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn pre(attr: TokenStream, file: TokenStream) -> TokenStream {
    let dummy_file: TokenStream2 = file.clone().into();
    proc_macro_error::set_dummy(quote! {
        #dummy_file
    });

    let mut file = parse_macro_input!(file as File);

    PreAttrVisitor::new(attr.into()).visit_file_mut(&mut file);

    let output = quote! {
        #file
    };

    // Reset the dummy here, in case errors were emitted in `render_pre`.
    // This will use the most up-to-date version of the generated code.
    proc_macro_error::set_dummy(quote! {
        #output
    });

    output.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn pre_defs_for(attr: TokenStream, module: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as DefinitionsForAttr);
    let module = parse_macro_input!(module as DefinitionsForModule);

    let output = module.render(attr);

    // Reset the dummy here, in case errors were emitted while generating the code.
    // This will use the most up-to-date version of the generated code.
    proc_macro_error::set_dummy(quote! {
        #output
    });

    output.into()
}
