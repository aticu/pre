//! Handles forwarding function calls to a different location.
//!
//! # What the generated code looks like
//!
//! ```rust,ignore
//! use std::ptr::read;
//!
//! #[pre::pre]
//! fn main() {
//!     unsafe {
//!         #[forward(pre_std::ptr)]
//!         #[assure(valid_ptr(src, r), reason = "a reference is a valid pointer")]
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
//!             #[assure(valid_ptr(src, r), reason = "a reference is a valid pointer")]
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
use proc_macro_error::{abort, emit_error};
use quote::{quote, quote_spanned};
use std::mem;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Pair,
    spanned::Spanned,
    Expr, ExprCall, ExprPath, Path, Token,
};

use crate::{call::Call, extern_crate::impl_block_stub_name};

/// The content of a `forward` attribute.
///
/// This specifies where the function the call should be forwarded to is located.
pub(crate) enum Forward {
    /// The given path should be added before the already present path.
    ///
    /// For a method, this is equivalent to an `impl` forward attribute.
    Direct {
        /// The path that should be added.
        path: Path,
        /// The span best representing the whole attribute.
        ///
        /// This is only optional, because it cannot be determined while parsing.
        /// It is filled immediately after parsing.
        span: Option<Span>,
    },
    /// The function or method to be called is located at the specified impl block.
    ImplBlock {
        /// The `impl` keyword that disambiguates this from a direct forward attribute.
        impl_keyword: Token![impl],
        /// The path to the impl block.
        path: Path,
        /// The span best representing the whole attribute.
        ///
        /// This is only optional, because it cannot be determined while parsing.
        /// It is filled immediately after parsing.
        span: Option<Span>,
    },
    /// The function to be called is found by replacing `from` with `to` in the path.
    Replace {
        /// The prefix of the path that should be replaced.
        from: Path,
        /// The arrow token that marks the replacement.
        _arrow: Token![->],
        /// The path that should be prepended instead of the removed prefix.
        to: Path,
        /// The span best representing the whole attribute.
        ///
        /// This is only optional, because it cannot be determined while parsing.
        /// It is filled immediately after parsing.
        span: Option<Span>,
    },
}

impl Parse for Forward {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let impl_keyword = if input.peek(Token![impl]) {
            Some(input.parse()?)
        } else {
            None
        };

        let first_path = input.parse()?;

        Ok(if input.is_empty() {
            if let Some(impl_keyword) = impl_keyword {
                Forward::ImplBlock {
                    impl_keyword,
                    path: first_path,
                    span: None,
                }
            } else {
                Forward::Direct {
                    path: first_path,
                    span: None,
                }
            }
        } else {
            let arrow = input.parse()?;
            let second_path = input.parse()?;

            Forward::Replace {
                from: first_path,
                _arrow: arrow,
                to: second_path,
                span: None,
            }
        })
    }
}

impl Spanned for Forward {
    fn span(&self) -> Span {
        match self {
            Forward::Direct { path, span } => span.unwrap_or_else(|| path.span()),
            Forward::ImplBlock {
                impl_keyword,
                path,
                span,
            } => span.unwrap_or_else(|| {
                impl_keyword
                    .span
                    .join(path.span())
                    .unwrap_or_else(|| path.span())
            }),
            Forward::Replace { from, to, span, .. } => {
                span.unwrap_or_else(|| from.span().join(to.span()).unwrap_or_else(|| to.span()))
            }
        }
    }
}

impl Forward {
    /// Updates the call to use the forwarded location.
    pub(super) fn update_call(self, mut call: Call, render: impl FnOnce(Call) -> Call) -> Expr {
        let original_call = call.clone();
        let span = self.span();

        match &mut call {
            Call::Function(ref mut fn_call) => {
                let fn_path = if let Expr::Path(p) = *fn_call.func.clone() {
                    p
                } else {
                    emit_error!(
                        fn_call.func,
                        "unable to determine at compile time which function is being called";
                        help = "use a direct path to the function instead"
                    );

                    return original_call.into();
                };

                parse2(match self {
                    Forward::Direct { .. } | Forward::Replace { .. } => {
                        mem::swap(
                            &mut *fn_call.func,
                            &mut Expr::Path(self.construct_new_path(&fn_path)),
                        );
                        let call = render(call);

                        quote_spanned! { span=>
                            if true {
                                #call
                            } else {
                                #original_call
                            }
                        }
                    }
                    Forward::ImplBlock { path, .. } => {
                        let fn_name = if let Some(segment) = fn_path.path.segments.last() {
                            &segment.ident
                        } else {
                            return original_call.into();
                        };

                        let rendered_call = render(create_empty_call(path, fn_name).into());

                        quote_spanned! { span=>
                            if true {
                                #original_call
                            } else {
                                #rendered_call;

                                unreachable!()
                            }
                        }
                    }
                })
                .expect("valid expression")
            }
            Call::Method(method_call) => match self {
                Forward::ImplBlock { path, .. } | Forward::Direct { path, .. } => {
                    let rendered_call = render(create_empty_call(path, &method_call.method).into());

                    parse2(quote_spanned! { span=>
                        if true {
                            #original_call
                        } else {
                            #rendered_call;

                            unreachable!()
                        }
                    })
                    .expect("valid expression")
                }
                Forward::Replace { ref to, .. } => {
                    emit_error!(
                        call.span(),
                        "a replacement `forward` attribute is not supported for method calls";
                        help = self.span() => "replace it with a direct location, such as {}", quote! { #to },
                    );

                    original_call.into()
                }
            },
        }
    }

    /// Constructs a new path correctly using addressing the forwarded function.
    pub(super) fn construct_new_path(self, fn_path: &ExprPath) -> ExprPath {
        let mut resulting_path = fn_path.clone();

        match self {
            Forward::Direct { ref path, .. } => {
                for (i, segment) in path.segments.iter().enumerate() {
                    resulting_path.path.segments.insert(i, segment.clone());
                }
            }
            Forward::ImplBlock { .. } => {
                unreachable!("`construct_new_path` is never called for an `impl` forward attribute")
            }
            Forward::Replace { from, to, .. } => {
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

    /// Sets the span of this `forward` attribute.
    pub(crate) fn set_span(&mut self, new_span: Span) {
        match self {
            Forward::Direct { span, .. }
            | Forward::ImplBlock { span, .. }
            | Forward::Replace { span, .. } => {
                span.replace(new_span);
            }
        }
    }
}

/// Creates an empty call to the given function.
fn create_empty_call(mut path: Path, fn_name: &impl std::fmt::Display) -> ExprCall {
    if let Some(segment_pair) = path.segments.pop() {
        path.segments
            .push(impl_block_stub_name(segment_pair.value(), fn_name, path.span()).into());
    } else {
        abort!(path, "path must have at least one segment");
    }

    ExprCall {
        attrs: Vec::new(),
        func: Box::new(
            ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            }
            .into(),
        ),
        paren_token: Default::default(),
        args: Default::default(),
    }
}

/// Checks if the path is a prefix and emits errors, if it isn't.
fn check_prefix(possible_prefix: &Path, path: &Path) -> bool {
    if possible_prefix.segments.len() > path.segments.len() {
        emit_error!(
            path,
            "cannot replace `{}` in this path",
            quote! { #possible_prefix };
            help = possible_prefix.span()=> "try specifing a prefix of `{}` in the `forward` attribute",
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
                help = possible_prefix.span()=> "try specifing a prefix of `{}` in the `forward` attribute",
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
