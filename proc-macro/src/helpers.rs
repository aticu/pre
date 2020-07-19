//! Allows retrieving the name of the main crate.

use lazy_static::lazy_static;
use proc_macro2::Span;
use proc_macro_error::abort_call_site;
use std::env;
use syn::{parse::Parse, spanned::Spanned, Attribute, Expr, Signature};

pub(crate) use attr::Attr;

mod attr;

/// The reason to display in examples on how to use reasons.
pub(crate) const HINT_REASON: &str = "<specify the reason why you can assure this here>";

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

/// Specifies what to do with a visited attribute.
pub(crate) enum AttributeAction {
    /// Remove the attribute from the resulting code.
    Remove,
    /// Keep the attribute in resulting code.
    Keep,
}

/// Visits all pre attributes of name `attr_name` and performs the `AttributeAction` on them.
pub(crate) fn visit_matching_attrs_parsed_mut<ParsedAttr: Parse + Spanned>(
    attributes: &mut Vec<Attribute>,
    attr_name: &str,
    mut visit: impl FnMut(Attr<ParsedAttr>) -> AttributeAction,
) -> Option<Span> {
    let mut span_of_all: Option<Span> = None;

    attributes.retain(|attr| match Attr::from_inner(attr_name, attr) {
        Some(attr) => {
            let span = attr.span();

            match visit(attr) {
                AttributeAction::Remove => {
                    span_of_all = Some(match span_of_all.take() {
                        Some(old_span) => old_span.join(span).unwrap_or_else(|| span),
                        None => span,
                    });

                    false
                }
                AttributeAction::Keep => true,
            }
        }
        None => true,
    });

    span_of_all
}

/// Visits all pre attributes of name `attr_name`.
pub(crate) fn visit_matching_attrs_parsed<ParsedAttr: Parse + Spanned>(
    attributes: &[Attribute],
    attr_name: &str,
    mut visit: impl FnMut(Attr<ParsedAttr>),
) {
    for attr in attributes {
        if let Some(attr) = Attr::from_inner(attr_name, attr) {
            visit(attr);
        }
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

/// Incorporates the given span into the signature.
///
/// Ideally both are shown, when the function definition is shown.
pub(crate) fn add_span_to_signature(span: Span, signature: &mut Signature) {
    signature.fn_token.span = signature.fn_token.span.join(span).unwrap_or_else(|| span);

    if let Some(token) = &mut signature.constness {
        token.span = token.span.join(span).unwrap_or_else(|| span);
    }

    if let Some(token) = &mut signature.asyncness {
        token.span = token.span.join(span).unwrap_or_else(|| span);
    }

    if let Some(token) = &mut signature.unsafety {
        token.span = token.span.join(span).unwrap_or_else(|| span);
    }

    if let Some(abi) = &mut signature.abi {
        abi.extern_token.span = abi.extern_token.span.join(span).unwrap_or_else(|| span);
    }
}
