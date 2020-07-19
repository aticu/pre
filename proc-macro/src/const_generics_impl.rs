//! Implements the procedural macros using a zero-sized const generics parameter.
//!
//! # Advantages of this approach
//! - helpful error messages for typos
//! - supports arbitrarily complex strings out of the box
//! - quick to compute
//!
//! # Disadvantages of this approach
//! - error messages for no invariants not very readable
//!
//! # What the generated code looks like
//!
//! ```rust,ignore
//! #[pre::pre(some_val > 42.0)]
//! fn has_preconditions(some_val: f32) -> f32 {
//!     assert!(some_val > 42.0);
//!
//!     some_val
//! }
//!
//! #[pre::pre]
//! fn main() {
//!     #[assure(some_val > 42.0, reason = "43.0 > 42.0")]
//!     has_preconditions(43.0);
//! }
//! ```
//!
//! turns into
//!
//! ```rust,ignore
//! #[doc = "..."]
//! fn has_preconditions(
//!     some_val: f32,
//!     #[cfg(not(doc))]
//!     _: ::core::marker::PhantomData<(::pre::BooleanCondition<"some_val > 42.0">,)>,
//! ) -> f32 {
//!     ::core::debug_assert!(
//!         some_val > 42.0
//!         "boolean precondition was wrongly assured: `{}`",
//!         ::core::stringify!(some_val > 42.0)
//!     );
//!     assert!(some_val > 42.0);
//!
//!     some_val
//! }
//!
//! fn main() {
//!     has_preconditions(
//!         43.0,
//!         ::core::marker::PhantomData::<(::pre::BooleanCondition<"some_val > 42.0">,)>,
//!     );
//! }
//! ```

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, TokenStreamExt};
use syn::{parse2, spanned::Spanned, Ident, ItemFn, LitStr};

use crate::{
    call::Call,
    helpers::{add_span_to_signature, CRATE_NAME},
    precondition::{CfgPrecondition, Precondition, ReadWrite},
};

/// Renders a precondition list to a token stream.
fn render_condition_list(mut preconditions: Vec<CfgPrecondition>, span: Span) -> TokenStream {
    preconditions.sort_unstable();

    let mut tokens = TokenStream::new();
    let crate_name = Ident::new(&CRATE_NAME, span);

    for precondition in preconditions {
        match precondition.precondition() {
            Precondition::ValidPtr {
                ident, read_write, ..
            } => {
                let ident_lit = LitStr::new(&ident.to_string(), ident.span());
                let rw_str = match read_write {
                    ReadWrite::Read { .. } => LitStr::new("r", read_write.span()),
                    ReadWrite::Write { .. } => LitStr::new("w", read_write.span()),
                    ReadWrite::Both { .. } => LitStr::new("r+w", read_write.span()),
                };
                tokens.append_all(quote_spanned! { precondition.span()=>
                    ::#crate_name::ValidPtrCondition::<#ident_lit, #rw_str>
                });
            }
            Precondition::ProperAlign { ident, .. } => {
                let ident_lit = LitStr::new(&ident.to_string(), ident.span());
                tokens.append_all(quote_spanned! { precondition.span()=>
                    ::#crate_name::ProperAlignCondition::<#ident_lit>
                });
            }
            Precondition::Boolean(expr) => {
                let as_str = LitStr::new(&quote! { #expr }.to_string(), precondition.span());

                tokens.append_all(quote_spanned! { precondition.span()=>
                    ::#crate_name::BooleanCondition::<#as_str>
                });
            }
            Precondition::Custom(string) => {
                tokens.append_all(quote_spanned! { precondition.span()=>
                    ::#crate_name::CustomCondition::<#string>
                });
            }
        }

        tokens.append_all(quote_spanned! { span=>
            ,
        });
    }

    tokens
}

/// Generates the code for the function with the precondition handling added.
pub(crate) fn render_pre(
    preconditions: Vec<CfgPrecondition>,
    function: &mut ItemFn,
    span: Span,
) -> TokenStream {
    let preconditions = render_condition_list(preconditions, span);

    // Include the precondition site into the span of the function.
    // This improves the error messages for the case where no preconditions are specified.
    add_span_to_signature(span, &mut function.sig);

    function.sig.inputs.push(
        parse2(quote_spanned! { span=>
            #[cfg(not(doc))]
            _: ::core::marker::PhantomData<(#preconditions)>
        })
        .expect("parses as a function argument"),
    );

    quote! {
        #function
    }
}

/// Generates the code for the call with the precondition handling added.
pub(crate) fn render_assure(
    preconditions: Vec<CfgPrecondition>,
    mut call: Call,
    span: Span,
) -> Call {
    let preconditions = render_condition_list(preconditions, span);

    call.args_mut().push(
        parse2(quote_spanned! { span=>
            ::core::marker::PhantomData::<(#preconditions)>
        })
        .expect("parses as an expression"),
    );

    call
}
