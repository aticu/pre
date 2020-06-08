//! Defines what a precondition is and how it's parsed.

use proc_macro2::Span;
use std::{cmp::Ordering, fmt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
    LitStr, Token,
};

pub(crate) use self::{kind::PreconditionKind, list::PreconditionList};

mod kind;
mod list;

/// The custom keywords used in preconditions.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(condition);
    custom_keyword!(reason);
}

/// A precondition for a function call.
pub(crate) struct Precondition {
    /// The `condition` keyword.
    _condition_keyword: custom_keywords::condition,
    /// The parentheses following the `condition` keyword.
    _parentheses: Paren,
    /// The kind of precondition.
    kind: PreconditionKind,
    /// The span before the reason.
    ///
    /// This is useful for reporting where to insert a reason in case its missing.
    before_reason_span: Span,
    /// The reason why the precondition holds.
    reason: Option<Reason>,
}

impl fmt::Debug for Precondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(reason) = &self.reason {
            write!(f, "condition({:?}, reason = {:?})", self.kind, reason)
        } else {
            write!(f, "condition({:?})", self.kind)
        }
    }
}

impl Parse for Precondition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Precondition {
            _condition_keyword: input.parse()?,
            _parentheses: parenthesized!(content in input),
            kind: content.parse()?,
            before_reason_span: content.span(),
            reason: Reason::parse(&content)?,
        })
    }
}

impl Precondition {
    /// Returns the kind of the precondition.
    pub(crate) fn kind(&self) -> &PreconditionKind {
        &self.kind
    }

    /// Returns the reason the precondition holds.
    pub(crate) fn reason(&self) -> Option<&LitStr> {
        self.reason.as_ref().map(|r| &r.reason)
    }

    /// Returns the span of precondition.
    pub(crate) fn span(&self) -> Span {
        self._condition_keyword
            .span()
            .join(
                self.reason
                    .as_ref()
                    .map(|r| r.span())
                    .unwrap_or_else(|| self._parentheses.span),
            )
            .unwrap_or_else(|| self._parentheses.span)
    }

    /// The span where to insert a reason, if it's missing.
    pub(crate) fn missing_reason_span(&self) -> Option<Span> {
        if self.reason.is_some() {
            None
        } else {
            Some(self.before_reason_span)
        }
    }
}

impl Ord for Precondition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind().cmp(other.kind())
    }
}

impl PartialOrd for Precondition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Precondition {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Precondition {}

/// A reason describing why a precondition holds.
struct Reason {
    /// The `,` separating the condition and the reason.
    _comma: Token![,],
    /// The `reason` keyword.
    _reason_keyword: custom_keywords::reason,
    /// The `=` separating the `reason` keyword and the reason.
    _eq: Token![=],
    /// The reason the precondition holds.
    reason: LitStr,
}

impl fmt::Debug for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.reason.value())
    }
}

impl Reason {
    /// Parses a reason for why a precondition holds.
    fn parse(input: ParseStream) -> syn::Result<Option<Self>> {
        if input.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self {
                _comma: input.parse()?,
                _reason_keyword: input.parse()?,
                _eq: input.parse()?,
                reason: input.parse()?,
            }))
        }
    }

    /// Returns the span of the reason.
    fn span(&self) -> Span {
        self._comma
            .span()
            .join(self.reason.span())
            .unwrap_or_else(|| self.reason.span())
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn parse_correct() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition("foo")
        });

        let result = result.expect("parsing should succeed");

        assert!(result.reason.is_none());
        assert!(result.reason().is_none());
    }

    #[test]
    fn parse_correct_with_reason() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition("foo", reason = "bar")
        });

        let result = result.expect("parsing should succeed");

        assert!(result.reason.is_some());
        assert!(result.reason().is_some());
    }

    #[test]
    fn parse_wrong_start() {
        let result: Result<Precondition, _> = parse2(quote! {
            ondition("foo")
        });

        assert!(result.is_err());
    }

    #[test]
    fn parse_no_parentheses() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition "foo"
        });

        assert!(result.is_err());
    }

    #[test]
    fn parse_comma_without_reason() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition("foo",)
        });

        assert!(result.is_err());
    }

    #[test]
    fn parse_reason_without_eq() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition("foo", reason)
        });

        assert!(result.is_err());
    }

    #[test]
    fn parse_reason_without_litstr() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition("foo", reason =)
        });

        assert!(result.is_err());
    }

    #[test]
    fn parse_reason_with_extra_tokens() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition("foo", reason = "bar" abc)
        });

        assert!(result.is_err());
    }

    #[test]
    fn parse_extra_tokens() {
        let result: Result<Precondition, _> = parse2(quote! {
            condition("foo", reason = "bar") abc
        });

        assert!(result.is_err());
    }
}
