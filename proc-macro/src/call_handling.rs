//! Functionality for parsing and visiting `assure` attributes.

use proc_macro2::Span;
use proc_macro_error::{emit_error, emit_warning};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Expr, LitStr, Token,
};

use self::def_statement::DefStatement;
use crate::{
    call::Call,
    helpers::{is_attr, visit_matching_attrs_parsed, Parenthesized},
    precondition::Precondition,
    render_assure,
};

mod def_statement;

/// The custom keywords used in the `assure` attribute.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(def);
    custom_keyword!(reason);
}

/// An `assure` declaration.
enum AssureAttr {
    /// Information where to find the definition of the preconditions.
    DefStatement(Parenthesized<DefStatement>),
    /// A statement that the precondition holds.
    Precondition(Parenthesized<PreconditionHoldsStatement>),
}

impl Parse for AssureAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);

        Ok(if content.peek(custom_keywords::def) {
            AssureAttr::DefStatement(Parenthesized::with_parentheses(parentheses, &content)?)
        } else {
            AssureAttr::Precondition(Parenthesized::with_parentheses(parentheses, &content)?)
        })
    }
}

/// A statement that a precondition holds.
enum PreconditionHoldsStatement {
    /// The statement had a reason attached to it.
    WithReason {
        /// The precondition that was stated.
        precondition: Precondition,
        /// The comma separating the precondition from the reason.
        _comma: Token![,],
        /// The reason that was stated.
        reason: Reason,
    },
    /// The statement written without a reason.
    WithoutReason {
        /// The precondition that was stated.
        precondition: Precondition,
        /// The span where to place the missing reason.
        missing_reason_span: Span,
    },
}

impl From<PreconditionHoldsStatement> for Precondition {
    fn from(holds_statement: PreconditionHoldsStatement) -> Precondition {
        match holds_statement {
            PreconditionHoldsStatement::WithoutReason { precondition, .. } => precondition,
            PreconditionHoldsStatement::WithReason { precondition, .. } => precondition,
        }
    }
}

impl Parse for PreconditionHoldsStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let precondition = input.parse()?;

        if input.is_empty() {
            Ok(PreconditionHoldsStatement::WithoutReason {
                precondition,
                missing_reason_span: input.span(),
            })
        } else {
            let comma = input.parse()?;
            let reason = input.parse()?;

            Ok(PreconditionHoldsStatement::WithReason {
                precondition,
                _comma: comma,
                reason,
            })
        }
    }
}

/// The reason why a precondition holds.
struct Reason {
    /// The `reason` keyword.
    _reason_keyword: custom_keywords::reason,
    /// The `=` separating the `reason` keyword and the reason.
    _eq: Token![=],
    /// The reason the precondition holds.
    reason: LitStr,
}

impl Parse for Reason {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let reason_keyword = input.parse()?;
        let eq = input.parse()?;
        let reason = input.parse()?;

        Ok(Reason {
            _reason_keyword: reason_keyword,
            _eq: eq,
            reason,
        })
    }
}

/// The reason to display in the hint where to add the reason.
const HINT_REASON: &str = "why does this hold?";

/// Renders the call, if necessary.
pub(crate) fn process_call(mut call: Call) -> Option<Expr> {
    let mut def_statement = None;
    let mut preconditions = Vec::new();

    let attr_span = visit_matching_attrs_parsed(
        call.attrs_mut(),
        |attr| is_attr("assure", attr),
        |parsed_attr| match parsed_attr {
            AssureAttr::DefStatement(Parenthesized { content: def, .. }) => {
                if let Some(old_def_statement) = def_statement.replace(def) {
                    let span = def_statement
                        .as_ref()
                        .expect("options contains a value, because it was just put there")
                        .span();
                    emit_error!(
                        span,
                        "duplicate `def(...)` statement";
                        help = old_def_statement.span() => "there can be just one definition site, try removing the wrong one"
                    );
                }
            }
            AssureAttr::Precondition(Parenthesized {
                content: precondition,
                ..
            }) => {
                preconditions.push(precondition);
            }
        },
    );

    if let Some(attr_span) = attr_span {
        Some(render_call(preconditions, def_statement, attr_span, call))
    } else {
        None
    }
}

/// Process a found `assure` attribute.
fn render_call(
    preconditions: Vec<PreconditionHoldsStatement>,
    def_statement: Option<DefStatement>,
    attr_span: Span,
    original_call: Call,
) -> Expr {
    let preconditions = check_reasons(preconditions);

    if let Some(def_statement) = def_statement {
        def_statement.update_call(original_call, |call| {
            render_assure(preconditions, call, attr_span)
        })
    } else {
        let output = render_assure(preconditions, original_call, attr_span);

        output.into()
    }
}

/// Checks that all reasons exist and make sense.
///
/// This function emits errors, if appropriate.
fn check_reasons(preconditions: Vec<PreconditionHoldsStatement>) -> Vec<Precondition> {
    for precondition in preconditions.iter() {
        match precondition {
            PreconditionHoldsStatement::WithReason { reason, .. } => {
                if let Some(reason) = unfinished_reason(&reason.reason) {
                    emit_warning!(
                        reason,
                        "you should specify a more meaningful reason here";
                        help = "specifying a meaningful reason here will help you and others understand why this is ok in the future"
                    )
                }
            }
            PreconditionHoldsStatement::WithoutReason {
                precondition,
                missing_reason_span,
                ..
            } => emit_error!(
                precondition.span(),
                "you need to specify a reason why this precondition holds";
                help = *missing_reason_span => "add `, reason = {:?}`", HINT_REASON
            ),
        }
    }

    preconditions
        .into_iter()
        .map(|holds_statement| holds_statement.into())
        .collect()
}

/// Returns an unfinished reason declaration for the precondition if one exists.
fn unfinished_reason(reason: &LitStr) -> Option<&LitStr> {
    let mut reason_val = reason.value();

    reason_val.make_ascii_lowercase();
    match &*reason_val {
        HINT_REASON | "todo" | "?" | "" => Some(reason),
        _ => None,
    }
}
