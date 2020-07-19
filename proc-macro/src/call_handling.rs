//! Functionality for parsing and visiting `assure` attributes.

use proc_macro2::Span;
use proc_macro_error::{emit_error, emit_warning};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Expr, LitStr, Token,
};

use self::forward::ForwardAttr;
use crate::{
    call::Call,
    helpers::{visit_matching_attrs_parsed_mut, Attr, AttributeAction, HINT_REASON},
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
    },
    /// The statement written without a reason.
    ///
    /// This is not permitted semantically.
    /// The only reason it is accepted syntactically is that it allows providing more relevant
    /// error messages.
    WithoutReason {
        /// The precondition that was stated.
        precondition: Precondition,
    },
}

impl From<AssureAttr> for Precondition {
    fn from(holds_statement: AssureAttr) -> Precondition {
        match holds_statement {
            AssureAttr::WithoutReason { precondition } => precondition,
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
                ..
            } => precondition
                .span()
                .join(reason.reason.span())
                .unwrap_or_else(|| precondition.span()),
            AssureAttr::WithoutReason { precondition } => precondition.span(),
        }
    }
}

impl Parse for AssureAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let precondition = input.parse()?;

        if input.is_empty() {
            Ok(AssureAttr::WithoutReason { precondition })
        } else {
            let comma = input.parse()?;
            let reason = input.parse()?;

            Ok(AssureAttr::WithReason {
                precondition,
                _comma: comma,
                reason,
            })
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

/// The attributes of a call expression.
pub(crate) struct CallAttributes {
    /// The span best representing all the attributes.
    pub(crate) span: Span,
    /// The optional `forward` attribute.
    pub(crate) forward: Option<Attr<ForwardAttr>>,
    /// The list of `assure` attributes.
    pub(crate) assure_attributes: Vec<Attr<AssureAttr>>,
}

/// Removes and returns all `pre`-related call-site attributes from the given attribute list.
pub(crate) fn remove_call_attributes(attributes: &mut Vec<Attribute>) -> Option<CallAttributes> {
    let mut forward = None;
    let mut assure_attributes = Vec::new();

    let preconditions_span = visit_matching_attrs_parsed_mut(attributes, "assure", |attr| {
        assure_attributes.push(attr);

        AttributeAction::Remove
    });

    let forward_span = visit_matching_attrs_parsed_mut(attributes, "forward", |attr| {
        let span = attr.span();

        if let Some(old_forward) = forward.replace(attr) {
            emit_error!(
                span,
                "duplicate `forward` attribute";
                help = old_forward.span() => "there can be just one location, try removing the wrong one"
            );
        }

        AttributeAction::Remove
    });

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
    check_reasons(&assure_attributes);

    let precondition = assure_attributes
        .into_iter()
        .map(|attr| attr.into())
        .collect();

    if let Some((forward, _, _)) = forward.map(|fwd| fwd.into_content()) {
        forward.update_call(original_call, |call| {
            render_assure(precondition, call, span)
        })
    } else {
        let output = render_assure(precondition, original_call, span);

        output.into()
    }
}

/// Checks that all reasons exist and make sense.
///
/// This function emits errors, if appropriate.
fn check_reasons(assure_attributes: &[Attr<AssureAttr>]) {
    for assure_attribute in assure_attributes.iter() {
        match assure_attribute.content() {
            AssureAttr::WithReason { reason, .. } => {
                if let Some(reason) = unfinished_reason(&reason.reason) {
                    emit_warning!(
                        reason,
                        "you should specify a different here";
                        help = "specifying a meaningful reason will help you and others understand why this is ok in the future"
                    )
                } else if reason.reason.value() == HINT_REASON {
                    let todo_help_msg = if cfg!(nightly) {
                        Some("using `TODO` here will emit a warning, reminding you to fix this later")
                    } else {
                        None
                    };

                    emit_error!(
                        reason.reason,
                        "you need to specify a different reason here";
                        help = "specifying a meaningful reason will help you and others understand why this is ok in the future";
                        help =? todo_help_msg
                    )
                }
            }
            AssureAttr::WithoutReason { precondition } => emit_error!(
                precondition.span(),
                "you need to specify a reason why this precondition holds";
                help = "add `, reason = {:?}`", HINT_REASON
            ),
        }
    }
}

/// Returns an unfinished reason declaration for the precondition if one exists.
fn unfinished_reason(reason: &LitStr) -> Option<&LitStr> {
    let mut reason_val = reason.value();

    reason_val.make_ascii_lowercase();
    match &*reason_val {
        "todo" | "?" | "" => Some(reason),
        _ => None,
    }
}
