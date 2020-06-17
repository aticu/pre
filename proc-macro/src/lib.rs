//! This crate contains the implementation for attributes used in the `pre` crate.
//!
//! Refer to the documentation of the `pre` crate for more information.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{emit_error, emit_warning, proc_macro_error};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    token::Paren,
    visit_mut::VisitMut,
    Item, ItemFn,
};

use crate::{
    assert_pre::AssertPreVisitor,
    def::{DefPreAttr, DefPreModule},
    precondition::Precondition,
};

mod assert_pre;
mod call;
mod def;
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

/// A `pre` attribute.
struct PreAttr {
    /// The parentheses surrounding the condition.
    _parentheses: Paren,
    /// The condition within the attribute.
    precondition: Precondition,
}

impl Parse for PreAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);
        let precondition = content.parse()?;

        Ok(PreAttr {
            _parentheses: parentheses,
            precondition,
        })
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn pre(attr: TokenStream, function: TokenStream) -> TokenStream {
    let dummy_function: TokenStream2 = function.clone().into();
    proc_macro_error::set_dummy(quote! {
        #dummy_function
    });

    let mut function = parse_macro_input!(function as ItemFn);

    let mut i = 0;
    let mut attrs = Vec::new();

    // TODO: Change this to drain_filter once it is stabilized
    // (see https://github.com/rust-lang/rust/issues/43244)
    while i < function.attrs.len() {
        if function.attrs[i].path.is_ident("pre") {
            attrs.push(function.attrs.remove(i));
        } else {
            i += 1;
        }
    }

    let mut preconditions = vec![parse_macro_input!(attr as Precondition)];
    let mut attr_span = preconditions[0].span();

    for attr in attrs {
        attr_span = attr_span.join(attr.span()).unwrap_or_else(|| attr.span());

        match syn::parse2::<PreAttr>(attr.tokens) {
            Ok(parsed_attr) => preconditions.push(parsed_attr.precondition),
            Err(err) => emit_error!(err),
        }
    }

    let output = render_pre(preconditions, function, attr_span);

    // Reset the dummy here, in case errors were emitted in `render_pre`.
    // This will use the most up-to-date version of the generated code.
    proc_macro_error::set_dummy(quote! {
        #output
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
