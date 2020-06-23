//! Handles specified alternative definition sites for functions.
//!
//! # What the generated code looks like
//!
//! ```rust,ignore
//! use std::ptr::read;
//!
//! #[pre::pre]
//! fn main() {
//!     unsafe {
//!         #[assert_pre(def(pre_std::ptr))]
//!         #[assert_pre(valid_ptr(src, r), reason = "a reference is a valid pointer")]
//!         read(&42);
//!     }
//! }
//! ```
//!
//! turns (roughly, if steps were not combined) into
//!
//! ```rust,ignore
//! use std::ptr::read;
//!
//! #[pre::pre]
//! fn main() {
//!     unsafe {
//!         if true {
//!             #[assert_pre(valid_ptr(src, r), reason = "a reference is a valid pointer")]
//!             pre_std::ptr::read(&42)
//!         } else {
//!             // To silence the unused import warnings.
//!             //
//!             // This should have the same type inference as the other call.
//!             read(&42)
//!         };
//!     }
//! }
//! ```

use proc_macro2::Span;
use proc_macro_error::emit_error;
use quote::quote;
use std::mem;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Pair,
    spanned::Spanned,
    Expr, ExprPath, Path, Token,
};

use crate::{call::Call, helpers::Parenthesized};

/// Provides information where to find the definition of the preconditions.
pub(super) struct DefStatement {
    /// The def keyword.
    def_keyword: super::custom_keywords::def,
    /// Information about the definition site.
    site: Parenthesized<DefStatementSite>,
}

impl Parse for DefStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let def_keyword = input.parse()?;
        let site = input.parse()?;

        Ok(DefStatement { def_keyword, site })
    }
}

impl Spanned for DefStatement {
    fn span(&self) -> Span {
        self.def_keyword
            .span()
            .join(self.site.parentheses.span)
            .unwrap_or_else(|| self.site.content.span())
    }
}

impl DefStatement {
    /// Updates the call to use the stored definition.
    pub(super) fn update_call(self, mut call: &mut Call) {
        match &mut call {
            Call::Function(ref mut call) => {
                if let Expr::Path(p) = *call.func.clone() {
                    mem::swap(
                        &mut *call.func,
                        &mut Expr::Path(self.construct_new_path(&p)),
                    );
                } else {
                    emit_error!(
                        call.func,
                        "unable to determine at compile time which function is being called";
                        help = "use a direct path to the function instead"
                    );
                }
            }
            Call::Method(_) => todo!(),
        }
    }

    /// Constructs a new path correctly using the new definition site.
    pub(super) fn construct_new_path(self, fn_path: &ExprPath) -> ExprPath {
        let mut resulting_path = fn_path.clone();

        match self.site.content {
            DefStatementSite::Direct { ref path } => {
                for (i, segment) in path.segments.iter().enumerate() {
                    resulting_path.path.segments.insert(i, segment.clone());
                }
            }
            DefStatementSite::Replace { from, to, .. } => {
                if !check_prefix(&from, &fn_path.path) {
                    return resulting_path;
                }

                resulting_path.path.segments = to
                    .segments
                    .into_pairs()
                    .map(punctuate_end) // we don't want to have an `End` in the middle
                    .chain(
                        resulting_path
                            .path
                            .segments
                            .into_pairs()
                            .skip(from.segments.len()),
                    )
                    .collect();

                // Make sure that the path doesn't end with `::`
                if let Some(last_value) = resulting_path.path.segments.pop() {
                    resulting_path.path.segments.push(last_value.into_value());
                }
            }
        }

        resulting_path
    }
}

/// Provides the definition in a `def(...)` statement.
enum DefStatementSite {
    /// The definition is found directly at the given path.
    Direct {
        /// The path where to find the definition.
        path: Path,
    },
    /// The definition is found by replacing `from` with `to` in the path.
    Replace {
        /// The path where the original function is found.
        from: Path,
        /// The arrow token that marks the replacement.
        _arrow: Token![->],
        /// The path where to function with the attached preconditions is found.
        to: Path,
    },
}

impl Parse for DefStatementSite {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first_path = input.parse()?;

        Ok(if input.is_empty() {
            DefStatementSite::Direct { path: first_path }
        } else {
            let arrow = input.parse()?;
            let second_path = input.parse()?;

            DefStatementSite::Replace {
                from: first_path,
                _arrow: arrow,
                to: second_path,
            }
        })
    }
}

impl Spanned for DefStatementSite {
    fn span(&self) -> Span {
        match self {
            DefStatementSite::Direct { path } => path.span(),
            DefStatementSite::Replace { from, to, .. } => {
                from.span().join(to.span()).unwrap_or_else(|| to.span())
            }
        }
    }
}

/// Checks if the path is a prefix and emits errors, if it isn't.
fn check_prefix(possible_prefix: &Path, path: &Path) -> bool {
    if possible_prefix.segments.len() > path.segments.len() {
        emit_error!(
            path,
            "cannot replace `{}` in this path",
            quote! { #possible_prefix };
            help = possible_prefix.span()=> "try specifing a prefix of `{}` in `def(...)`",
            quote! { #path }
        );
        return false;
    }

    for (prefix_segment, path_segment) in possible_prefix.segments.iter().zip(path.segments.iter())
    {
        if prefix_segment != path_segment {
            emit_error!(
                path,
                "cannot replace `{}` in this path",
                quote! { #possible_prefix };
                note = path_segment.span()=> "`{}` != `{}`",
                quote! { #prefix_segment },
                quote! { #path_segment };
                help = possible_prefix.span()=> "try specifing a prefix of `{}` in `def(...)`",
                quote! { #path }
            );
            return false;
        }
    }

    true
}

/// Transforms `Pair::End` pairs to `Pair::Punctuated` ones.
fn punctuate_end<T, P: Default>(pair: Pair<T, P>) -> Pair<T, P> {
    match pair {
        Pair::End(end) => Pair::Punctuated(end, Default::default()),
        Pair::Punctuated(elem, punct) => Pair::Punctuated(elem, punct),
    }
}
