//! Defines the different kinds of preconditions.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::{cmp::Ordering, fmt};
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
    Error, Expr, Ident, LitStr, Token,
};

/// The custom keywords used by the precondition kinds.
mod custom_keywords {
    use syn::custom_keyword;

    custom_keyword!(valid_ptr);
    custom_keyword!(proper_align);
    custom_keyword!(r);
    custom_keyword!(w);
}

/// The different kinds of preconditions.
#[derive(Clone)]
pub(crate) enum Precondition {
    /// Requires that the given pointer is valid.
    ValidPtr {
        /// The `valid_ptr` keyword.
        valid_ptr_keyword: custom_keywords::valid_ptr,
        /// The parentheses following the `valid_ptr` keyword.
        parentheses: Paren,
        /// The identifier of the pointer.
        ident: Ident,
        /// The comma between the identifier and the read/write information.
        _comma: Token![,],
        /// Information on what accesses of the pointer must be valid.
        read_write: ReadWrite,
    },
    ProperAlign {
        /// The `proper_align` keyword.
        proper_align_keyword: custom_keywords::proper_align,
        /// The parentheses following the `proper_align` keyword.
        parentheses: Paren,
        /// The identifier of the pointer.
        ident: Ident,
    },
    /// An expression that should evaluate to a boolean value.
    Boolean(Box<Expr>),
    /// A custom precondition that is spelled out in a string.
    Custom(LitStr),
}

impl fmt::Display for Precondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Precondition::ValidPtr {
                ident, read_write, ..
            } => write!(f, "valid_ptr({}, {})", ident.to_string(), read_write),
            Precondition::ProperAlign { ident, .. } => {
                write!(f, "proper_align({})", ident.to_string())
            }
            Precondition::Boolean(expr) => write!(f, "{}", quote! { #expr }),
            Precondition::Custom(lit) => write!(f, "{:?}", lit.value()),
        }
    }
}

/// Parses an identifier that is valid for use in a precondition.
fn parse_precondition_ident(input: ParseStream) -> syn::Result<Ident> {
    let lookahead = input.lookahead1();

    if lookahead.peek(Token![self]) {
        input.call(Ident::parse_any)
    } else if lookahead.peek(Ident) {
        input.parse()
    } else {
        Err(lookahead.error())
    }
}

impl Parse for Precondition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start_span = input.span();

        if input.peek(custom_keywords::valid_ptr) {
            let valid_ptr_keyword = input.parse()?;
            let content;
            let parentheses = parenthesized!(content in input);
            let ident = parse_precondition_ident(&content)?;
            let comma = content.parse()?;
            let read_write = content.parse()?;

            if content.is_empty() {
                Ok(Precondition::ValidPtr {
                    valid_ptr_keyword,
                    parentheses,
                    ident,
                    _comma: comma,
                    read_write,
                })
            } else {
                Err(content.error("unexpected token"))
            }
        } else if input.peek(custom_keywords::proper_align) {
            let proper_align_keyword = input.parse()?;
            let content;
            let parentheses = parenthesized!(content in input);
            let ident = parse_precondition_ident(&content)?;

            if content.is_empty() {
                Ok(Precondition::ProperAlign {
                    proper_align_keyword,
                    parentheses,
                    ident,
                })
            } else {
                Err(content.error("unexpected token"))
            }
        } else if input.peek(LitStr) {
            Ok(Precondition::Custom(input.parse()?))
        } else {
            let expr = input.parse();

            match expr {
                Ok(expr) => Ok(Precondition::Boolean(Box::new(expr))),
                Err(mut err) => {
                    err.combine(Error::new(
                        start_span,
                        "expected `valid_ptr`, `proper_align`, a string literal or a boolean expression",
                    ));

                    Err(err)
                }
            }
        }
    }
}

impl Spanned for Precondition {
    fn span(&self) -> Span {
        match self {
            Precondition::ValidPtr {
                valid_ptr_keyword,
                parentheses,
                ..
            } => valid_ptr_keyword
                .span()
                .join(parentheses.span)
                .unwrap_or_else(|| valid_ptr_keyword.span()),
            Precondition::ProperAlign {
                proper_align_keyword,
                parentheses,
                ..
            } => proper_align_keyword
                .span()
                .join(parentheses.span)
                .unwrap_or_else(|| proper_align_keyword.span()),
            Precondition::Boolean(expr) => expr.span(),
            Precondition::Custom(lit) => lit.span(),
        }
    }
}

impl Precondition {
    /// Returns a unique id for each descriminant.
    fn descriminant_id(&self) -> usize {
        match self {
            Precondition::ValidPtr { .. } => 0,
            Precondition::ProperAlign { .. } => 1,
            Precondition::Boolean(_) => 2,
            Precondition::Custom(_) => 3,
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
                Precondition::ValidPtr {
                    ident: ident_self, ..
                },
                Precondition::ValidPtr {
                    ident: ident_other, ..
                },
            ) => ident_self.cmp(ident_other),
            (
                Precondition::ProperAlign {
                    ident: ident_self, ..
                },
                Precondition::ProperAlign {
                    ident: ident_other, ..
                },
            ) => ident_self.cmp(ident_other),
            (Precondition::Boolean(expr_self), Precondition::Boolean(expr_other)) => {
                quote!(#expr_self)
                    .to_string()
                    .cmp(&quote!(#expr_other).to_string())
            }
            (Precondition::Custom(lit_self), Precondition::Custom(lit_other)) => {
                lit_self.value().cmp(&lit_other.value())
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

impl ReadWrite {
    /// Generates a short description suitable for usage in generated documentation.
    ///
    /// The generated description should finish the sentence
    /// "The pointer must be valid for...".
    pub(crate) fn doc_description(&self) -> &str {
        match self {
            ReadWrite::Read { .. } => "reads",
            ReadWrite::Write { .. } => "writes",
            ReadWrite::Both { .. } => "reads and writes",
        }
    }
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
                .unwrap_or(r_keyword.span),
        }
    }
}

/// A precondition with an optional `cfg` applying to it.
pub(crate) struct CfgPrecondition {
    /// The precondition with additional data.
    pub(crate) precondition: Precondition,
    /// The `cfg` applying to the precondition.
    #[allow(dead_code)]
    pub(crate) cfg: Option<TokenStream>,
    /// The span best representing the precondition.
    pub(crate) span: Span,
}

impl CfgPrecondition {
    /// The raw precondition.
    pub(crate) fn precondition(&self) -> &Precondition {
        &self.precondition
    }
}

impl Spanned for CfgPrecondition {
    fn span(&self) -> Span {
        self.span
    }
}

impl PartialEq for CfgPrecondition {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal)
    }
}

impl Eq for CfgPrecondition {}

impl PartialOrd for CfgPrecondition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CfgPrecondition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.precondition.cmp(&other.precondition)
    }
}

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
    fn parse_wrong_expr() {
        {
            let result: Result<Precondition, _> = parse2(quote! {
                a ++ b
            });
            assert!(result.is_err());
        }

        {
            let result: Result<Precondition, _> = parse2(quote! {
                17 - + -- + []
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
