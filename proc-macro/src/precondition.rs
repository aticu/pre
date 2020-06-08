//! Defines what a precondition is and how it's parsed.

use std::{cmp::Ordering, fmt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
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
            reason: Reason::parse(&content)?,
        })
    }
}

impl Precondition {
    /// Returns the kind of the precondition.
    pub(crate) fn kind(&self) -> &PreconditionKind {
        &self.kind
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
pub(crate) struct Reason {
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
}
