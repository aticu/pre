//! Allows retrieving the name of the main crate.

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_error::{abort_call_site, emit_error};
use quote::quote_spanned;
use std::env;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    token::Paren,
    Attribute, Expr, Signature, Token,
};

use crate::precondition::CfgPrecondition;

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

/// Combines the `cfg` of all preconditions if possible.
pub(crate) fn combine_cfg(preconditions: &[CfgPrecondition], _span: Span) -> Option<TokenStream> {
    const MISMATCHED_CFG: &str = "mismatched `cfg` predicates for preconditions";
    const MISMATCHED_CFG_NOTE: &str =
        "all preconditions must have syntactically equal `cfg` predicates";

    let render_cfg = |cfg: Option<&TokenStream>| cfg.map(|cfg| format!("{}", cfg));

    let first_cfg = preconditions.first().and_then(|p| p.cfg.clone());
    let first_cfg_rendered = render_cfg(first_cfg.as_ref());

    for precondition in preconditions.iter().skip(1) {
        if first_cfg_rendered != render_cfg(precondition.cfg.as_ref()) {
            match (&first_cfg, &precondition.cfg) {
                (Some(first_cfg), Some(current_cfg)) => {
                    emit_error!(
                        current_cfg.span(),
                        MISMATCHED_CFG;
                        note = MISMATCHED_CFG_NOTE;
                        note = first_cfg.span() => "`{}` != `{}`", first_cfg, current_cfg
                    );
                }
                (Some(cfg), None) | (None, Some(cfg)) => {
                    emit_error!(
                        cfg.span(),
                        MISMATCHED_CFG;
                        note = MISMATCHED_CFG_NOTE;
                        note = "some preconditions have a `cfg` predicate and some do not"
                    );
                }
                (None, None) => unreachable!("two `None`s are equal to each other"),
            }
        }
    }

    first_cfg
}

/// A `TokenStream` surrounded by parentheses.
struct Parenthesized {
    /// The parentheses surrounding the `TokenStream`.
    parentheses: Paren,
    /// The content that was surrounded by the parentheses.
    content: TokenStream,
}

impl Parse for Parenthesized {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);

        Ok(Parenthesized {
            parentheses,
            content: content.parse()?,
        })
    }
}

/// Parses the token stream to the next comma and returns the result as a new token stream.
fn parse_to_comma(input: &mut TokenStream) -> (TokenStream, Option<Token![,]>) {
    let mut to_comma = TokenStream::new();
    let mut comma = None;

    let mut token_iter = input.clone().into_iter();

    loop {
        match token_iter.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == ',' => {
                comma = Some(
                    parse2(TokenTree::from(p).into()).expect("`,` token tree is parsed as a comma"),
                );

                break;
            }
            Some(token_tree) => to_comma.extend(std::iter::once(token_tree)),
            None => break,
        }
    }

    *input = TokenStream::new();
    input.extend(token_iter);

    (to_comma, comma)
}

/// Transforms multiple attributes in a single `cfg_attr` into multiple `cfg_attr`.
///
/// ```rust,ignore
/// #[cfg_attr(some_cfg, attr_1, attr_2, /* ..., */ attr_n)]
/// ```
///
/// becomes
///
/// ```rust,ignore
/// #[cfg_attr(some_cfg, attr_1)]
/// #[cfg_attr(some_cfg, attr_2)]
/// /* ... */
/// #[cfg_attr(some_cfg, attr_n)]
/// ```
pub(crate) fn flatten_cfgs(attributes: &mut Vec<Attribute>) {
    let mut i = 0;
    while i < attributes.len() {
        if attributes[i].path.is_ident("cfg_attr") {
            let attribute = attributes.remove(i);

            let (parentheses, mut input) = if let Ok(Parenthesized {
                parentheses,
                content: input,
            }) = parse2(attribute.tokens.clone())
            {
                (parentheses, input)
            } else {
                // Ignore invalid `cfg_attr`
                attributes.insert(i, attribute);
                i += 1;
                continue;
            };

            let (cfg, comma) = parse_to_comma(&mut input);
            if comma.is_none() {
                // Ignore invalid `cfg_attr`
                attributes.insert(i, attribute);
                i += 1;
                continue;
            }

            // This ensures that the attributes are added in the correct order (the order they were
            // specified in).
            //
            // Note that i is kept at the same value to handle nested `cfg_attr` attributes (e.g.
            // `#[cfg_attr(abc, cfg_attr(def, ...))]`)
            loop {
                let (attr_tokens, comma) = parse_to_comma(&mut input);

                let new_attribute = Attribute {
                    pound_token: attribute.pound_token,
                    style: attribute.style,
                    bracket_token: attribute.bracket_token,
                    path: attribute.path.clone(),
                    tokens: quote_spanned! { parentheses.span=>
                        (#cfg, #attr_tokens)
                    },
                };

                attributes.insert(i, new_attribute);
                i += 1;

                if comma.is_none() {
                    // We found the last attribute
                    break;
                }
            }
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_cfg_flattening() {
        let mut transformed_func: syn::ItemFn = syn::parse_quote! {
            #[cfg_attr(all(target_endian = "little", target_endian = "big"), attr1, attr2, attr3)]
            #[cfg_attr(all(target_endian = "big", target_endian = "little"), attr4, attr5, attr6)]
            fn foo() {}
        };

        flatten_cfgs(&mut transformed_func.attrs);

        let desired_result: syn::ItemFn = syn::parse_quote! {
            #[cfg_attr(all(target_endian = "little", target_endian = "big"), attr1)]
            #[cfg_attr(all(target_endian = "little", target_endian = "big"), attr2)]
            #[cfg_attr(all(target_endian = "little", target_endian = "big"), attr3)]
            #[cfg_attr(all(target_endian = "big", target_endian = "little"), attr4)]
            #[cfg_attr(all(target_endian = "big", target_endian = "little"), attr5)]
            #[cfg_attr(all(target_endian = "big", target_endian = "little"), attr6)]
            fn foo() {}
        };

        assert_eq!(transformed_func, desired_result);
    }
}
