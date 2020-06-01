//! Defines what a precondition is and how it's parsed.

use quote::format_ident;
use std::fmt;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    Ident, LitStr, Token,
};

pub(crate) use self::kind::PreconditionKind;

mod kind;

/// The custom keywords used in preconditions.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(condition);
    custom_keyword!(holds);
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
}

impl fmt::Debug for Precondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "condition({:?})", self.kind)
    }
}

impl Parse for Precondition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Precondition {
            _condition_keyword: input.parse()?,
            _parentheses: parenthesized!(content in input),
            kind: content.parse()?,
        })
    }
}

impl Precondition {
    /// Renders this precondition as a `String` representing an identifier.
    pub(crate) fn render_as_ident(&self) -> Ident {
        /// Escapes characters that are not valid in identifiers.
        fn escape_non_ident_chars(string: String) -> String {
            string
                .chars()
                .map(|c| match c {
                    '_' | '0'..='9' | 'a'..='z' | 'A'..='Z' => c.to_string(),
                    other => format!("_{:x}", other as u32),
                })
                .collect()
        }

        match self.kind.clone() {
            PreconditionKind::ValidPtr { ident, .. } => format_ident!("_valid_ptr_{}", ident),
            PreconditionKind::Custom(string) => {
                format_ident!("_custom_{}", escape_non_ident_chars(string.value()))
            }
        }
    }
}

/// A declaration that a precondition holds.
pub(crate) struct PreconditionHolds {
    /// The `condition` keyword.
    _condition_keyword: custom_keywords::holds,
    /// The parentheses following the `condition` keyword.
    _parentheses: Paren,
    /// The kind of precondition.
    kind: PreconditionKind,
    /// The `,` separating the condition and the reason.
    _comma: Token![,],
    /// The `reason` keyword.
    _reason_keyword: custom_keywords::reason,
    /// The `=` separating the `reason` keyword and the reason.
    _eq: Token![=],
    /// The reason the precondition holds.
    reason: LitStr,
}

impl fmt::Debug for PreconditionHolds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "holds({:?}, reason = {})",
            self.kind,
            self.reason.value()
        )
    }
}

impl Parse for PreconditionHolds {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;

        Ok(PreconditionHolds {
            _condition_keyword: input.parse()?,
            _parentheses: parenthesized!(content in input),
            kind: content.parse()?,
            _comma: content.parse()?,
            _reason_keyword: content.parse()?,
            _eq: content.parse()?,
            reason: content.parse()?,
        })
    }
}

impl PreconditionHolds {
    /// Renders this precondition as a `String` representing an identifier.
    pub(crate) fn render_as_ident(&self) -> Ident {
        /// Escapes characters that are not valid in identifiers.
        fn escape_non_ident_chars(string: String) -> String {
            string
                .chars()
                .map(|c| match c {
                    '_' | '0'..='9' | 'a'..='z' | 'A'..='Z' => c.to_string(),
                    other => format!("_{:x}", other as u32),
                })
                .collect()
        }

        match self.kind.clone() {
            PreconditionKind::ValidPtr { ident, .. } => format_ident!("_valid_ptr_{}", ident),
            PreconditionKind::Custom(string) => {
                format_ident!("_custom_{}", escape_non_ident_chars(string.value()))
            }
        }
    }
}
