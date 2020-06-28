//! Functionality for parsing and visiting `assure` attributes.

use proc_macro2::Span;
use proc_macro_error::{emit_error, emit_warning};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Expr, LitStr, Token,
};

use self::forward::Forward;
use crate::{
    call::Call,
    helpers::{is_attr, visit_matching_attrs_parsed, Parenthesized},
    precondition::Precondition,
    render_assure,
};

mod forward;

/// The custom keywords used in the `assure` attribute.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(reason);
}

/// An attribute with an assurance that a precondition holds.
pub(crate) enum AssureAttr {
    /// The statement had a reason attached to it.
    WithReason {
        /// The precondition that was stated.
        precondition: Precondition,
        /// The comma separating the precondition from the reason.
        _comma: Token![,],
        /// The reason that was stated.
        reason: Reason,
        /// The span best representing the whole attribute.
        ///
        /// This is only optional, because it cannot be determined while parsing.
        /// It is filled immediately after parsing.
        span: Option<Span>,
    },
    /// The statement written without a reason.
    ///
    /// This is not permitted semantically.
    /// The only reason it is accepted syntactically is that it allows providing more relevant
    /// error messages.
    WithoutReason {
        /// The precondition that was stated.
        precondition: Precondition,
        /// The span where to place the missing reason.
        missing_reason_span: Span,
        /// The span best representing the whole attribute.
        ///
        /// This is only optional, because it cannot be determined while parsing.
        /// It is filled immediately after parsing.
        span: Option<Span>,
    },
}

impl From<AssureAttr> for Precondition {
    fn from(holds_statement: AssureAttr) -> Precondition {
        match holds_statement {
            AssureAttr::WithoutReason { precondition, .. } => precondition,
            AssureAttr::WithReason { precondition, .. } => precondition,
        }
    }
}

impl Spanned for AssureAttr {
    fn span(&self) -> Span {
        match self {
            AssureAttr::WithReason {
                precondition,
                reason,
                span,
                ..
            } => span.unwrap_or_else(|| {
                precondition
                    .span()
                    .join(reason.reason.span())
                    .unwrap_or_else(|| precondition.span())
            }),
            AssureAttr::WithoutReason {
                precondition, span, ..
            } => span.unwrap_or_else(|| precondition.span()),
        }
    }
}

impl Parse for AssureAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let precondition = input.parse()?;

        if input.is_empty() {
            Ok(AssureAttr::WithoutReason {
                precondition,
                missing_reason_span: input.span(),
                span: None,
            })
        } else {
            let comma = input.parse()?;
            let reason = input.parse()?;

            Ok(AssureAttr::WithReason {
                precondition,
                _comma: comma,
                reason,
                span: None,
            })
        }
    }
}

impl AssureAttr {
    /// Sets the span of this `assure` attribute.
    fn set_span(&mut self, new_span: Span) {
        match self {
            AssureAttr::WithReason { span, .. } | AssureAttr::WithoutReason { span, .. } => {
                span.replace(new_span);
            }
        }
    }
}

/// The reason why a precondition holds.
pub(crate) struct Reason {
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

/// The attributes of a call expression.
pub(crate) struct CallAttributes {
    /// The span best representing all the attributes.
    pub(crate) span: Span,
    /// The optional `forward` attribute.
    pub(crate) forward: Option<Forward>,
    /// The list of `assure` attributes.
    pub(crate) assure_attributes: Vec<AssureAttr>,
}

/// Removes and returns all `pre`-related call-site attributes from the given attribute list.
pub(crate) fn remove_call_attributes(attributes: &mut Vec<Attribute>) -> Option<CallAttributes> {
    let mut forward = None;
    let mut assure_attributes = Vec::new();

    let preconditions_span = visit_matching_attrs_parsed(
        attributes,
        |attr| is_attr("assure", attr),
        |Parenthesized {
             content: mut assure_attribute,
             ..
         }: Parenthesized<AssureAttr>,
         span| {
            assure_attribute.set_span(span);

            assure_attributes.push(assure_attribute);
        },
    );
    let forward_span = visit_matching_attrs_parsed(
        attributes,
        |attr| is_attr("forward", attr),
        |Parenthesized {
             content: mut fwd, ..
         }: Parenthesized<Forward>,
         span| {
            fwd.set_span(span);

            if let Some(old_forward) = forward.replace(fwd) {
                emit_error!(
                    span,
                    "duplicate `forward` attribute";
                    help = old_forward.span() => "there can be just one location, try removing the wrong one"
                );
            }
        },
    );

    let span = match (preconditions_span, forward_span) {
        (Some(preconditions_span), Some(forward_span)) => Some(
            preconditions_span
                .join(forward_span)
                .unwrap_or_else(|| preconditions_span),
        ),
        (Some(span), None) => Some(span),
        (None, Some(span)) => Some(span),
        (None, None) => None,
    };

    if let Some(span) = span {
        Some(CallAttributes {
            span,
            forward,
            assure_attributes,
        })
    } else {
        None
    }
}

/// Renders the call using the found attributes for it.
pub(crate) fn render_call(
    CallAttributes {
        span,
        forward,
        assure_attributes,
    }: CallAttributes,
    original_call: Call,
) -> Expr {
    let preconditions = check_reasons(assure_attributes);

    if let Some(forward) = forward {
        forward.update_call(original_call, |call| {
            render_assure(preconditions, call, span)
        })
    } else {
        let output = render_assure(preconditions, original_call, span);

        output.into()
    }
}

/// Checks that all reasons exist and make sense.
///
/// This function emits errors, if appropriate.
fn check_reasons(assure_attributes: Vec<AssureAttr>) -> Vec<Precondition> {
    for assure_attribute in assure_attributes.iter() {
        match assure_attribute {
            AssureAttr::WithReason { reason, .. } => {
                if let Some(reason) = unfinished_reason(&reason.reason) {
                    emit_warning!(
                        reason,
                        "you should specify a more meaningful reason here";
                        help = "specifying a meaningful reason here will help you and others understand why this is ok in the future"
                    )
                }
            }
            AssureAttr::WithoutReason {
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

    assure_attributes
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
