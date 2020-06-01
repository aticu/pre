//! Implements the procedural macros using a zero-sized const generics parameter.
//!
//! # Advantages of this approach
//! - helpful error messages for typos
//! - supports arbitrarily complex strings out of the box
//! - quick to compute
//!
//! # Disadvantages of this approach
//! - error messages for no invariants not very readable

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, ExprCall, ItemFn, LitStr};

use crate::precondition::{Precondition, PreconditionHolds, PreconditionKind};

impl ToTokens for Precondition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.kind() {
            PreconditionKind::Custom(string) => {
                tokens.append_all(quote! {
                    impl ::pre::CustomCondition::<#string>,
                });
            }
            PreconditionKind::ValidPtr { ident, .. } => {
                let ident_lit = LitStr::new(&ident.to_string(), ident.span());
                tokens.append_all(quote! {
                    impl ::pre::ValidPtrCondition::<#ident_lit>,
                });
            }
        }
    }
}

/// Generates the code for the function with the precondition handling added.
pub(crate) fn render_pre(preconditions: Precondition, mut function: ItemFn) -> TokenStream {
    function.sig.inputs.push(parse_quote! {
        _: ::core::marker::PhantomData<(#preconditions)>
    });

    quote! {
        #function
    }
}

impl ToTokens for PreconditionHolds {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.kind() {
            PreconditionKind::Custom(string) => {
                tokens.append_all(quote! {
                    ::pre::CustomConditionHolds::<#string>,
                });
            }
            PreconditionKind::ValidPtr { ident, .. } => {
                let ident_lit = LitStr::new(&ident.to_string(), ident.span());
                tokens.append_all(quote! {
                    ::pre::ValidPtrConditionHolds::<#ident_lit>,
                });
            }
        }
    }
}

/// Generates the code for the call with the precondition handling added.
pub(crate) fn render_assert_precondition(
    preconditions: PreconditionHolds,
    mut call: ExprCall,
) -> TokenStream {
    call.args.push(parse_quote! {
        ::core::marker::PhantomData::<(#preconditions)>
    });

    quote! {
        #call
    }
}
