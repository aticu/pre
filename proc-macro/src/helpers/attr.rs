//! An abstraction for various types of attributes.

use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro_error::emit_error;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    token::Paren,
    Attribute, Path, Token,
};

use super::CRATE_NAME;
use crate::precondition::{CfgPrecondition, Precondition};

/// Checks if the given attribute is an `attr_to_check` attribute of the main crate.
fn is_attr(attr_to_check: &str, path: &Path) -> bool {
    if path.is_ident(attr_to_check) {
        true
    } else if path.segments.len() == 2 {
        // Note that `Path::leading_colon` is not checked here, so paths both with and without a
        // leading colon are accepted here
        path.segments[0].ident == *CRATE_NAME && path.segments[1].ident == attr_to_check
    } else {
        false
    }
}

/// A `TokenStream` surrounded by parentheses.
struct Parenthesized {
    /// The parentheses surrounding the `TokenStream`.
    parentheses: Paren,
    /// The content that was surrounded by the parentheses.
    content: TokenStream,
}

impl Parse for Parenthesized {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let parentheses = parenthesized!(content in input);

        Ok(Parenthesized {
            parentheses,
            content: content.parse()?,
        })
    }
}

/// A `Path` followed by parentheses surrounding a `TokenStream`.
struct PathAndParenthesized {
    /// The path at the beginning of the construct.
    path: Path,
    /// The parentheses surrounding the `TokenStream`.
    parentheses: Paren,
    /// The content that was surrounded by the parentheses.
    content: TokenStream,
}

impl Parse for PathAndParenthesized {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        let content;
        let parentheses = parenthesized!(content in input);

        Ok(PathAndParenthesized {
            path,
            parentheses,
            content: content.parse()?,
        })
    }
}

/// Represents an attribute as seen by pre.
pub(crate) enum Attr<Content> {
    /// The attribute is contained in an `cfg_attr`.
    ///
    /// This occurs when parsing inner `cfg_attr` wrapped attributes.
    ///
    /// Example of what the parser sees: `#[cfg_attr(some_condition, path(content))]`
    WithCfg {
        /// The `cfg_attr` keyword.
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///   ^^^^^^^^
        /// ```
        _cfg_attr_keyword: Path,
        /// The outer parentheses of the attribute.
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///           ^                             ^
        /// ```
        _outer_parentheses: Paren,
        /// The configuration of the attribute.
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///            ^^^^^^^^^^^^^^
        /// ```
        cfg: TokenStream,
        /// The comma separating the configuration from the actual attribute.
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///                          ^
        /// ```
        _comma: Token![,],
        /// The path of the actual attribute.
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///                            ^^^^
        /// ```
        _path: Path,
        /// The parentheses of the actual attribute.
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///                                ^       ^
        /// ```
        _inner_parentheses: Paren,
        /// The content of the attribute.
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///                                 ^^^^^^^
        /// ```
        content: Content,
        /// The span best representing the inner attribute.
        ///
        /// Ideally this is
        ///
        /// ```text
        /// #[cfg_attr(some_condition, path(content))]
        ///                            ^^^^^^^^^^^^^
        /// ```
        span: Span,
    },
    /// The attribute is contained within parentheses.
    ///
    /// This occurs when parsing inner attributes.
    ///
    /// Example of what the parser sees: `#[path(content)]`
    WithParen {
        /// The path of the attribute.
        ///
        /// ```text
        /// #[path(content)]
        ///   ^^^^
        /// ```
        _path: Path,
        /// The parentheses of the attribute.
        ///
        /// ```text
        /// #[path(content)]
        ///       ^       ^
        /// ```
        _parentheses: Paren,
        /// The content of the attribute.
        ///
        /// ```text
        /// #[path(content)]
        ///        ^^^^^^^
        /// ```
        content: Content,
        /// The span best representing the attribute.
        ///
        /// Ideally this is
        ///
        /// ```text
        /// #[path(content)]
        /// ^^^^^^^^^^^^^^^^
        /// ```
        span: Span,
    },
    /// The attribute can be parsed directly.
    ///
    /// This occurs when parsing an attribute as a proc macro input.
    ///
    /// Example of what the parser sees: `content`
    Direct {
        /// The content of the attribute.
        content: Content,
    },
}

impl<Content: Parse + Spanned> Attr<Content> {
    /// Creates a parsed attribute from an attribute seen inside of a proc macro invocation.
    pub(crate) fn from_inner(target_attr: &str, attribute: &Attribute) -> Option<Attr<Content>> {
        if is_attr(target_attr, &attribute.path) {
            let Parenthesized {
                parentheses,
                content,
            } = parse2(attribute.tokens.clone())
                .map_err(|err| emit_error!(err))
                .ok()?;

            Some(Attr::WithParen {
                _path: attribute.path.clone(),
                _parentheses: parentheses,
                content: parse2(content).map_err(|err| emit_error!(err)).ok()?,
                span: attribute
                    .pound_token
                    .span
                    .join(attribute.bracket_token.span)
                    .unwrap_or_else(|| attribute.bracket_token.span),
            })
        } else if attribute.path.is_ident("cfg_attr") {
            let Parenthesized {
                parentheses: outer_parentheses,
                content: cfg_attr_content,
            } = parse2(attribute.tokens.clone())
                .map_err(|err| emit_error!(err))
                .ok()?;

            let mut cfg = TokenStream::new();
            let comma;

            let mut cfg_content_iter = cfg_attr_content.into_iter();

            let rest_tokens = loop {
                match cfg_content_iter.next()? {
                    TokenTree::Punct(p) if p.as_char() == ',' => {
                        let as_token_tree: TokenTree = p.into();

                        comma = parse2(as_token_tree.into())
                            .expect("`,` token tree is parsed as a comma");

                        let mut rest_tokens = TokenStream::new();
                        rest_tokens.extend(cfg_content_iter);
                        break rest_tokens;
                    }
                    token_tree => cfg.extend(std::iter::once(token_tree)),
                }
            };

            let PathAndParenthesized {
                path,
                parentheses: inner_parentheses,
                content,
            } = parse2(rest_tokens).map_err(|err| emit_error!(err)).ok()?;

            if !is_attr(target_attr, &path) {
                return None;
            }

            let span = path
                .span()
                .join(inner_parentheses.span)
                .unwrap_or_else(|| inner_parentheses.span);

            Some(Attr::WithCfg {
                _cfg_attr_keyword: attribute.path.clone(),
                _outer_parentheses: outer_parentheses,
                cfg,
                _comma: comma,
                _path: path,
                _inner_parentheses: inner_parentheses,
                content: parse2(content).map_err(|err| emit_error!(err)).ok()?,
                span,
            })
        } else {
            None
        }
    }

    /// Accesses the content of this attribute.
    pub(crate) fn content(&self) -> &Content {
        match self {
            Attr::WithCfg { content, .. } => content,
            Attr::WithParen { content, .. } => content,
            Attr::Direct { content } => content,
        }
    }

    /// Returns the pieces necessary to create a `CfgPrecondition` manually.
    pub(crate) fn into_content(self) -> (Content, Option<TokenStream>, Span) {
        match self {
            Attr::WithCfg {
                content, cfg, span, ..
            } => (content, Some(cfg), span),
            Attr::WithParen { content, span, .. } => (content, None, span),
            Attr::Direct { content } => {
                let span = content.span();

                (content, None, span)
            }
        }
    }
}

impl<Content: Spanned> Spanned for Attr<Content> {
    fn span(&self) -> Span {
        match self {
            Attr::WithCfg { span, .. } => *span,
            Attr::WithParen { span, .. } => *span,
            Attr::Direct { content } => content.span(),
        }
    }
}

impl<Content> From<Content> for Attr<Content> {
    fn from(content: Content) -> Self {
        Attr::Direct { content }
    }
}

impl<Content: Into<Precondition> + Spanned> From<Attr<Content>> for CfgPrecondition {
    fn from(val: Attr<Content>) -> Self {
        match val {
            Attr::WithCfg {
                content, span, cfg, ..
            } => CfgPrecondition {
                precondition: content.into(),
                cfg: Some(cfg),
                span,
            },
            Attr::WithParen { content, span, .. } => CfgPrecondition {
                precondition: content.into(),
                cfg: None,
                span,
            },
            Attr::Direct { content } => {
                let span = content.span();

                CfgPrecondition {
                    precondition: content.into(),
                    cfg: None,
                    span,
                }
            }
        }
    }
}
