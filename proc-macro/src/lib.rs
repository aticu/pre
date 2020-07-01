//! This crate contains the implementation for attributes used in the `pre` crate.
//!
//! Refer to the documentation of the `pre` crate for more information.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, File};

use crate::pre_attr::PreAttrVisitor;

mod call;
mod call_handling;
mod documentation;
mod extern_crate;
mod helpers;
mod pre_attr;
mod precondition;

cfg_if::cfg_if! {
    if #[cfg(nightly)] {
        mod const_generics_impl;
        pub(crate) use crate::const_generics_impl::{render_assure, render_pre};
    } else {
        mod struct_impl;
        pub(crate) use crate::struct_impl::{render_assure, render_pre};
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
pub fn assure(_: TokenStream, _: TokenStream) -> TokenStream {
    // This macro currently only has two purposes:
    // - Exist as a place to put documentation for the actual `assure` attribute, which is
    // implemented inside the `pre` attribute.
    // - Emit an error with a more helpful message than "attribute not found", if the user uses
    // `assure` in the wrong place.
    abort_call_site!(
        "this attribute by itself is currently non-functional";
        help = "use it on an expression in an item wrapped by a `pre` attribute"
    )
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn extern_crate(attr: TokenStream, module: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as extern_crate::Attr);
    let module = parse_macro_input!(module as extern_crate::Module);

    let output = module.render(attr);

    // Reset the dummy here, in case errors were emitted while generating the code.
    // This will use the most up-to-date version of the generated code.
    proc_macro_error::set_dummy(quote! {
        #output
    });

    output.into()
}
