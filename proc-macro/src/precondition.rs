//! Defines the different kinds of preconditions.

use proc_macro2::Span;
use std::{cmp::Ordering, fmt};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
    Ident, LitStr, Token,
};

/// The custom keywords used by the precondition kinds.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(valid_ptr);
    custom_keyword!(r);
    custom_keyword!(w);
}

/// Whether something is readable, writable or both.
#[derive(Clone)]
pub(crate) enum ReadWrite {
    /// The described thing is only readable.
    Read {
        /// The `r` keyword, indicating readability.
        r_keyword: custom_keywords::r,
    },
    /// The described thing is only writable.
    Write {
        /// The `w` keyword, indicating writability.
        w_keyword: custom_keywords::w,
    },
    /// The described thing is both readable and writable.
    Both {
        /// The `r` keyword, indicating readability.
        r_keyword: custom_keywords::r,
        /// The `+` between the `r` and the `w`, if both are present.
        _plus: Token![+],
        /// The `w` keyword, indicating writability.
        w_keyword: custom_keywords::w,
    },
}

impl fmt::Display for ReadWrite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReadWrite::Read { .. } => write!(f, "r"),
            ReadWrite::Write { .. } => write!(f, "w"),
            ReadWrite::Both { .. } => write!(f, "r+w"),
        }
    }
}

impl Parse for ReadWrite {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(custom_keywords::w) {
            Ok(ReadWrite::Write {
                w_keyword: input.parse()?,
            })
        } else if lookahead.peek(custom_keywords::r) {
            let r_keyword = input.parse()?;

            if input.peek(Token![+]) {
                let plus = input.parse()?;
                let w_keyword = input.parse()?;

                Ok(ReadWrite::Both {
                    r_keyword,
                    _plus: plus,
                    w_keyword,
                })
            } else {
                Ok(ReadWrite::Read { r_keyword })
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl Spanned for ReadWrite {
    fn span(&self) -> Span {
        match self {
            ReadWrite::Read { r_keyword } => r_keyword.span,
            ReadWrite::Write { w_keyword } => w_keyword.span,
            ReadWrite::Both {
                r_keyword,
                w_keyword,
                ..
            } => r_keyword
                .span
                .join(w_keyword.span)
                .unwrap_or_else(|| r_keyword.span),
        }
    }
}

/// The different kinds of preconditions.
#[derive(Clone)]
pub(crate) enum Precondition {
    /// Requires that the given pointer is valid.
    ValidPtr {
        /// The `valid_ptr` keyword.
        valid_ptr_keyword: custom_keywords::valid_ptr,
        /// The parentheses following the `valid_ptr` keyword.
        _parentheses: Paren,
        /// The identifier of the pointer.
        ident: Ident,
        /// The comma between the identifier and the read/write information.
        _comma: Token![,],
        /// Information on what accesses of the pointer must be valid.
        read_write: ReadWrite,
    },
    /// A custom precondition that is spelled out in a string.
    Custom(LitStr),
}

impl fmt::Display for Precondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Precondition::ValidPtr {
                ident, read_write, ..
            } => write!(f, "valid_ptr({}, {})", ident.to_string(), read_write),
            Precondition::Custom(lit) => write!(f, "{:?}", lit.value()),
        }
    }
}

impl Parse for Precondition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(custom_keywords::valid_ptr) {
            let valid_ptr_keyword = input.parse()?;
            let content;
            let parentheses = parenthesized!(content in input);
            let ident = content.parse()?;
            let comma = content.parse()?;
            let read_write = content.parse()?;

            if content.is_empty() {
                Ok(Precondition::ValidPtr {
                    valid_ptr_keyword,
                    _parentheses: parentheses,
                    ident,
                    _comma: comma,
                    read_write,
                })
            } else {
                Err(content.error("unexpected token"))
            }
        } else if lookahead.peek(LitStr) {
            Ok(Precondition::Custom(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Spanned for Precondition {
    fn span(&self) -> Span {
        match self {
            Precondition::Custom(lit) => lit.span(),
            Precondition::ValidPtr {
                valid_ptr_keyword,
                read_write,
                ..
            } => valid_ptr_keyword
                .span()
                .join(read_write.span())
                .unwrap_or_else(|| valid_ptr_keyword.span()),
        }
    }
}

impl Precondition {
    /// Returns a unique id for each descriminant.
    fn descriminant_id(&self) -> usize {
        match self {
            Precondition::ValidPtr { .. } => 0,
            Precondition::Custom(_) => 1,
        }
    }
}

// Define an order for the preconditions here.
//
// The exact ordering does not really matter, as long as it is deterministic.
impl Ord for Precondition {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Precondition::ValidPtr { ident: ident_a, .. },
                Precondition::ValidPtr { ident: ident_b, .. },
            ) => ident_a.to_string().cmp(&ident_b.to_string()),
            (Precondition::Custom(lit_a), Precondition::Custom(lit_b)) => {
                lit_a.value().cmp(&lit_b.value())
            }
            _ => {
                debug_assert_ne!(self.descriminant_id(), other.descriminant_id());

                self.descriminant_id().cmp(&other.descriminant_id())
            }
        }
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

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn parse_correct_custom() {
        let result: Result<Precondition, _> = parse2(quote! {
            "foo"
        });
        assert!(result.is_ok());
    }

    #[test]
    fn parse_correct_valid_ptr() {
        {
            let result: Result<Precondition, _> = parse2(quote! {
                valid_ptr(foo, r)
            });
            assert!(result.is_ok());
        }

        {
            let result: Result<Precondition, _> = parse2(quote! {
                valid_ptr(foo, r+w)
            });
            assert!(result.is_ok());
        }

        {
            let result: Result<Precondition, _> = parse2(quote! {
                valid_ptr(foo, w)
            });
            assert!(result.is_ok());
        }
    }

    #[test]
    fn parse_unknown_keyword() {
        {
            let result: Result<Precondition, _> = parse2(quote! {
                unknown_keyword
            });
            assert!(result.is_err());
        }

        {
            let result: Result<Precondition, _> = parse2(quote! {
                unknown_keyword("abc")
            });
            assert!(result.is_err());
        }
    }

    #[test]
    fn parse_extra_tokens() {
        {
            let result: Result<Precondition, _> = parse2(quote! {
                "foo" bar
            });
            assert!(result.is_err());
        }

        {
            let result: Result<Precondition, _> = parse2(quote! {
                valid_ptr(foo, r+w+x)
            });
            assert!(result.is_err());
        }
    }
}
