//! Allows retrieving the name of the main crate.

use proc_macro2::Span;
use proc_macro_error::{abort_call_site, emit_error};
use std::env;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
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

/// Removes matching attributes, parses them, and then allows visiting them.
///
/// This returns the most appropriate span to reference the original attributes.
pub(crate) fn visit_matching_attrs_parsed<ParsedAttr: Parse>(
    attributes: &mut Vec<Attribute>,
    mut filter: impl FnMut(&mut Attribute) -> bool,
    mut visit: impl FnMut(ParsedAttr),
) -> Option<Span> {
    let mut attr_span: Option<Span> = None;
    let mut i = 0;

    // TODO: use `drain_filter` once it's stabilized (see
    // https://github.com/rust-lang/rust/issues/43244).
    while i < attributes.len() {
        if filter(&mut attributes[i]) {
            let attr = attributes.remove(i);

            attr_span = Some(match attr_span.take() {
                Some(old_span) => old_span.join(attr.span()).unwrap_or_else(|| attr.span()),
                None => attr.span(),
            });

            match syn::parse2::<ParsedAttr>(attr.tokens) {
                Ok(parsed_attr) => visit(parsed_attr),
                Err(err) => emit_error!(err),
            }
        } else {
            i += 1;
        }
    }

    attr_span
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
