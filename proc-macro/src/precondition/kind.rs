//! Defines the different kinds of preconditions.

use std::{cmp::Ordering, fmt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
    Ident, LitStr,
};

/// The custom keywords used by the precondition kinds.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(valid_ptr);
}

/// The different kinds of preconditions.
#[derive(Clone)]
pub(crate) enum PreconditionKind {
    /// Requires that the given pointer is valid.
    ValidPtr {
        /// The `valid_ptr` keyword.
        _valid_ptr_keyword: custom_keywords::valid_ptr,
        /// The parentheses following the `valid_ptr` keyword.
        _parentheses: Paren,
        /// The identifier of the pointer.
        ident: Ident,
    },
    /// A custom precondition that is spelled out in a string.
    Custom(LitStr),
}

impl fmt::Debug for PreconditionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreconditionKind::ValidPtr {
                _valid_ptr_keyword: _,
                _parentheses: _,
                ident,
            } => write!(f, "valid_ptr({})", ident.to_string()),
            PreconditionKind::Custom(lit) => write!(f, "{:?}", lit.value()),
        }
    }
}

impl Parse for PreconditionKind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(custom_keywords::valid_ptr) {
            let content;

            Ok(PreconditionKind::ValidPtr {
                _valid_ptr_keyword: input.parse()?,
                _parentheses: parenthesized!(content in input),
                ident: content.parse()?,
            })
        } else if lookahead.peek(LitStr) {
            Ok(PreconditionKind::Custom(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl PreconditionKind {
    /// Returns a unique id for each descriminant.
    fn descriminant_id(&self) -> usize {
        match self {
            PreconditionKind::ValidPtr { .. } => 0,
            PreconditionKind::Custom(_) => 1,
        }
    }
}

// Define an order for the preconditions here.
//
// The exact ordering does not really matter, as long as it is deterministic.
impl Ord for PreconditionKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                PreconditionKind::ValidPtr { ident: ident_a, .. },
                PreconditionKind::ValidPtr { ident: ident_b, .. },
            ) => ident_a.to_string().cmp(&ident_b.to_string()),
            (PreconditionKind::Custom(lit_a), PreconditionKind::Custom(lit_b)) => {
                lit_a.value().cmp(&lit_b.value())
            }
            _ => {
                debug_assert_ne!(self.descriminant_id(), other.descriminant_id());

                self.descriminant_id().cmp(&other.descriminant_id())
            }
        }
    }
}

impl PartialOrd for PreconditionKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PreconditionKind {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for PreconditionKind {}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn parse_correct_custom() {
        let result: Result<PreconditionKind, _> = parse2(quote! {
            "foo"
        });

        assert!(result.is_ok());
    }

    #[test]
    fn parse_correct_valid_ptr() {
        let result: Result<PreconditionKind, _> = parse2(quote! {
            valid_ptr(foo)
        });

        assert!(result.is_ok());
    }

    #[test]
    fn parse_unknown_keyword() {
        let result: Result<PreconditionKind, _> = parse2(quote! {
            unknown_keyword
        });

        assert!(result.is_err());

        let result: Result<PreconditionKind, _> = parse2(quote! {
            unknown_keyword("abc")
        });

        assert!(result.is_err());
    }

    #[test]
    fn parse_extra_tokens() {
        let result: Result<PreconditionKind, _> = parse2(quote! {
            "foo" bar
        });

        assert!(result.is_err());
    }
}
