//! Functionality for parsing and visiting `assert_pre` attributes.

use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use std::mem;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Pair,
    spanned::Spanned,
    token::Paren,
    visit_mut::VisitMut,
    Attribute, Expr, ExprCall, ExprPath, LitStr, Path, Token,
};

use crate::{
    precondition::{Precondition, PreconditionList},
    render_assert_pre,
};

/// The custom keywords used in the `assert_pre` attribute.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(def);
}

/// An `assert_pre` declaration.
pub(crate) struct AssertPreAttr {
    /// The parentheses surrounding the attribute.
    _parentheses: Paren,
    /// Information where to find the definition of the preconditions.
    def_statement: Option<DefStatement>,
    /// The precondition list in the declaration.
    preconditions: PreconditionList<Precondition>,
}

impl Parse for AssertPreAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);

        let def_statement = if content.peek(custom_keywords::def) {
            Some(content.parse()?)
        } else {
            None
        };

        let preconditions = content.parse()?;

        Ok(AssertPreAttr {
            _parentheses: parentheses,
            def_statement,
            preconditions,
        })
    }
}

/// Provides information where to find the definition of the preconditions.
struct DefStatement {
    /// The def keyword.
    _def_keyword: custom_keywords::def,
    /// The parentheses surrounding the definition site.
    _parentheses: Paren,
    /// Information about the definition site.
    site: DefStatementSite,
    /// The comma following the `def(...)` statement.
    _comma: Token![,],
}

impl Parse for DefStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let def_keyword = input.parse()?;
        let content;
        let parentheses = parenthesized!(content in input);
        let site = content.parse()?;
        let comma = input.parse()?;

        Ok(DefStatement {
            _def_keyword: def_keyword,
            _parentheses: parentheses,
            site,
            _comma: comma,
        })
    }
}

impl DefStatement {
    /// Constructs a new path correctly using the new definition site.
    fn construct_new_path(&self, fn_path: &ExprPath) -> ExprPath {
        let mut resulting_path = fn_path.clone();

        match self.site {
            DefStatementSite::Direct { ref path } => {
                for (i, segment) in path.segments.iter().enumerate() {
                    resulting_path.path.segments.insert(i, segment.clone());
                }
            }
            DefStatementSite::Replace {
                ref from, ref to, ..
            } => {
                if from.segments.len() > resulting_path.path.segments.len() {
                    emit_error!(
                        fn_path,
                        "cannot replace `{}` in this path",
                        quote! { #from };
                        help = from.span()=> "try specifing a prefix of `{}` in the `def(...)`",
                        quote! { #fn_path }
                    );
                    return resulting_path;
                }

                for (from_segment, fn_segment) in from
                    .segments
                    .iter()
                    .zip(resulting_path.path.segments.iter())
                {
                    if from_segment != fn_segment {
                        emit_error!(
                            fn_path,
                            "cannot replace `{}` in this path",
                            quote! { #from };
                            note = fn_segment.span()=> "`{}` != `{}`",
                            quote! { #from_segment },
                            quote! { #fn_segment };
                            help = from.span()=> "try specifing a prefix of `{}` in the `def(...)`",
                            quote! { #fn_path }
                        );
                        return resulting_path.clone();
                    }
                }

                resulting_path.path.segments = to
                    .segments
                    .pairs()
                    .map(|pair| match pair {
                        Pair::Punctuated(segment, punct) => {
                            Pair::Punctuated(segment.clone(), punct.clone())
                        }
                        Pair::End(segment) => Pair::Punctuated(segment.clone(), Default::default()),
                    })
                    .chain(
                        resulting_path
                            .path
                            .segments
                            .pairs()
                            .map(|pair| match pair {
                                Pair::Punctuated(segment, punct) => {
                                    Pair::Punctuated(segment.clone(), punct.clone())
                                }
                                Pair::End(segment) => Pair::End(segment.clone()),
                            })
                            .skip(from.segments.len()),
                    )
                    .collect();
            }
        }

        resulting_path
    }
}

/// Provides the definition in a `def(...)` statement.
enum DefStatementSite {
    /// The definition is found directly at the given path.
    Direct {
        /// The path where to find the definition.
        path: Path,
    },
    /// The definition is found by replacing `from` with `to` in the path.
    Replace {
        /// The path where the original function is found.
        from: Path,
        /// The arrow token that marks the replacement.
        _arrow: Token![->],
        /// The path where to function with the attached preconditions is found.
        to: Path,
    },
}

impl Parse for DefStatementSite {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first_path = input.parse()?;

        Ok(if input.is_empty() {
            DefStatementSite::Direct { path: first_path }
        } else {
            let arrow = input.parse()?;
            let second_path = input.parse()?;

            DefStatementSite::Replace {
                from: first_path,
                _arrow: arrow,
                to: second_path,
            }
        })
    }
}

/// The reason to display in the hint where to add the reason.
const HINT_REASON: &'static str = "why does this hold?";

/// The name of the macro used to assert that a condition holds.
const ASSERT_CONDITION_HOLDS_ATTR: &'static str = "assert_pre";

/// A visitor for `assert_pre` declarations.
pub(crate) struct AssertPreVisitor;

impl VisitMut for AssertPreVisitor {
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        let mut i = 0;
        let mut attr_found = false;
        while i < call.attrs.len() {
            if call.attrs[i].path.is_ident(ASSERT_CONDITION_HOLDS_ATTR) {
                let attr = call.attrs.remove(i);

                if attr_found {
                    emit_error!(
                        attr,
                        "duplicate {} attribute found",
                        ASSERT_CONDITION_HOLDS_ATTR;
                        hint = "combine the list of conditions into one attribute"
                    );
                    continue;
                } else {
                    attr_found = true;
                }

                if let Ok(parsed_attr) =
                    syn::parse2(attr.tokens.clone()).map_err(|err| emit_error!(err))
                {
                    process_attribute(parsed_attr, attr, call);
                }
            } else {
                i += 1;
            }
        }

        syn::visit_mut::visit_expr_call_mut(self, call);
    }
}

/// Returns an unfinished reason declaration for the precondition if one exists.
fn unfinished_reason(precondition: &Precondition) -> Option<&LitStr> {
    let reason = precondition.reason().map(|r| r.value());

    if let Some(mut reason) = reason {
        reason.make_ascii_lowercase();
        match &*reason {
            HINT_REASON | "todo" | "?" => precondition.reason(),
            _ => None,
        }
    } else {
        None
    }
}

/// Process a found `assert_pre` attribute.
fn process_attribute(attr: AssertPreAttr, original_attr: Attribute, call: &mut ExprCall) {
    for precondition in attr.preconditions.iter() {
        if precondition.reason().is_none() {
            let missing_reason_span = precondition
                .missing_reason_span()
                .expect("the reason is missing");
            emit_error!(
                precondition.span(),
                "you need to specify a reason why this precondition holds";
                help = missing_reason_span => "add `, reason = {:?}`", HINT_REASON
            );
        } else if let Some(reason) = unfinished_reason(precondition) {
            emit_warning!(
                reason,
                "you should specify a more meaningful reason here";
                help = "specifying a meaningful reason here will help you and others understand why this is ok in the future"
            )
        }
    }

    if let Some(def_statement) = attr.def_statement {
        if let Expr::Path(p) = *call.func.clone() {
            let mut new_path = Expr::Path(def_statement.construct_new_path(&p));

            mem::swap(&mut *call.func, &mut new_path);
        } else {
            emit_error!(
                call.func,
                "unable to determine at compile time which function is being called";
                help = "use a direct path to the function instead"
            );
        }
    }

    let attr_span = original_attr
        .pound_token
        .span
        .join(original_attr.bracket_token.span)
        .unwrap_or_else(|| original_attr.bracket_token.span);
    let mut output = render_assert_pre(attr.preconditions, call.clone(), attr_span);

    mem::swap(&mut output, call);
}
