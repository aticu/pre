//! Allows retrieving the name of the main crate.

use proc_macro2::Span;
use proc_macro_error::abort_call_site;
use std::env;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    Attribute, Ident,
};

/// Returns the name of the main crate.
pub(crate) fn crate_name() -> Ident {
    let name = match proc_macro_crate::crate_name("pre") {
        Ok(name) => name,
        Err(err) => match env::var("CARGO_PKG_NAME") {
            // This allows for writing documentation tests on the functions themselves.
            //
            // This *may* lead to false positives, if someone also names their crate `pre`, however
            // it will very likely fail to compile at a later stage then.
            Ok(val) if val == "pre" => "pre".into(),
            _ => abort_call_site!("crate `pre` must be imported: {}", err),
        },
    };
    Ident::new(&name, Span::call_site())
}

/// Removes all attributes where `filter` returns true from `attributes` and returns them.
///
/// This function effectively fulfills the same purpose as `Vec::drain_filter` and can be removed,
/// once that is [stablized](https://github.com/rust-lang/rust/issues/43244).
pub(crate) fn remove_matching_attrs(
    attributes: &mut Vec<Attribute>,
    mut filter: impl FnMut(&mut Attribute) -> bool,
) -> Vec<Attribute> {
    let mut i = 0;
    let mut removed_attributes = Vec::new();

    while i < attributes.len() {
        if filter(&mut attributes[i]) {
            removed_attributes.push(attributes.remove(i));
        } else {
            i += 1;
        }
    }

    removed_attributes
}

/// A parsable thing surrounded by parentheses.
pub(crate) struct Parenthesized<T> {
    /// The parentheses surrounding the object.
    _parentheses: Paren,
    /// The content that was surrounded by the parentheses.
    pub(crate) content: T,
}

impl<T: Parse> Parenthesized<T> {
    /// Parses the content, if the parentheses were already parsed.
    pub(crate) fn with_parentheses(parentheses: Paren, input: ParseStream) -> syn::Result<Self> {
        Ok(Parenthesized {
            _parentheses: parentheses,
            content: input.parse()?,
        })
    }
}

impl<T: Parse> Parse for Parenthesized<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);
        let content = content.parse()?;

        Ok(Parenthesized {
            _parentheses: parentheses,
            content,
        })
    }
}
