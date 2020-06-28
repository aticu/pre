//! Allows retrieving the name of the main crate.

use lazy_static::lazy_static;
use proc_macro2::Span;
use proc_macro_error::{abort_call_site, emit_error};
use std::env;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
    Attribute, Expr,
};

lazy_static! {
    /// Returns the name of the main `pre` crate.
    pub(crate) static ref CRATE_NAME: String = {
        match proc_macro_crate::crate_name("pre") {
            Ok(name) => name,
            Err(err) => match env::var("CARGO_PKG_NAME") {
                // This allows for writing documentation tests on the functions themselves.
                //
                // This *may* lead to false positives, if someone also names their crate `pre`, however
                // it will very likely fail to compile at a later stage then.
                Ok(val) if val == "pre" => "pre".into(),
                _ => abort_call_site!("crate `pre` must be imported: {}", err),
            },
        }
    };
}

/// Checks if the given attribute is an `attr_to_check` attribute of the main crate.
pub(crate) fn is_attr(attr_to_check: &str, attr: &Attribute) -> bool {
    let path = &attr.path;

    if path.is_ident(attr_to_check) {
        true
    } else if path.segments.len() == 2 {
        // Note that `Path::leading_colon` is not checked here, so paths both with and without a
        // leading colon are accepted here
        path.segments[0].ident == *CRATE_NAME && path.segments[1].ident == attr_to_check
    } else {
        false
    }
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

/// Returns the attributes of the given expression.
pub(crate) fn attributes_of_expression(expr: &mut Expr) -> Option<&mut Vec<Attribute>> {
    macro_rules! extract_attributes_from {
        ($expr:expr => $($variant:ident),*) => {
            match $expr {
                $(
                Expr::$variant(e) => Some(&mut e.attrs),
                )*
                _ => None,
            }
        }
    }

    extract_attributes_from!(expr =>
        Array, Assign, AssignOp, Async, Await, Binary, Block, Box, Break, Call, Cast,
        Closure, Continue, Field, ForLoop, Group, If, Index, Let, Lit, Loop, Macro, Match,
        MethodCall, Paren, Path, Range, Reference, Repeat, Return, Struct, Try, TryBlock, Tuple,
        Type, Unary, Unsafe, While, Yield
    )
}
