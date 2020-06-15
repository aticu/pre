//! Implements the procedural macros using a zero-sized const generics parameter.
//!
//! # Advantages of this approach
//! - helpful error messages for typos
//! - supports arbitrarily complex strings out of the box
//! - quick to compute
//!
//! # Disadvantages of this approach
//! - error messages for no invariants not very readable

use proc_macro2::{Span, TokenStream};
use proc_macro_crate::crate_name;
use proc_macro_error::abort_call_site;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use std::env;
use syn::{parse2, parse_quote, spanned::Spanned, Ident, ItemFn, LitStr};

use crate::{
    call::Call,
    precondition::{kind::ReadWrite, Precondition, PreconditionKind, PreconditionList},
};

/// Returns the name of the main crate.
fn get_crate_name() -> Ident {
    let name = match crate_name("pre") {
        Ok(name) => name,
        Err(err) => match env::var("CARGO_PKG_NAME") {
            // This allows for writing documentation tests on the functions themselves.
            //
            // This *may* lead to false positives, if someone also names their crate `pre`, however
            // it will very likely fail to compile at a later stage then.
            Ok(val) if val == "pre" => "pre".into(),
            _ => abort_call_site!("crate `pre` must be imported: {}", err),
        },
    };
    Ident::new(&name, Span::call_site())
}

impl<T: ToTokens + Ord> ToTokens for PreconditionList<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for precondition in self.sorted_iter() {
            tokens.append_all(quote! {
                #precondition,
            });
        }
    }
}

impl ToTokens for Precondition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pre = get_crate_name();
        match self.kind() {
            PreconditionKind::Custom(string) => {
                tokens.append_all(quote_spanned! { self.span()=>
                    ::#pre::CustomConditionHolds::<#string>
                });
            }
            PreconditionKind::ValidPtr {
                ident, read_write, ..
            } => {
                let ident_lit = LitStr::new(&ident.to_string(), ident.span());
                let rw_str = match read_write {
                    ReadWrite::Read { .. } => LitStr::new("r", read_write.span()),
                    ReadWrite::Write { .. } => LitStr::new("w", read_write.span()),
                    ReadWrite::Both { .. } => LitStr::new("r+w", read_write.span()),
                };
                tokens.append_all(quote_spanned! { self.span()=>
                    ::#pre::ValidPtrConditionHolds::<#ident_lit, #rw_str>
                });
            }
        }
    }
}

/// Generates the code for the function with the precondition handling added.
pub(crate) fn render_pre(
    preconditions: PreconditionList<Precondition>,
    mut function: ItemFn,
) -> TokenStream {
    function.sig.inputs.push(parse_quote! {
        _: ::core::marker::PhantomData<(#preconditions)>
    });

    quote! {
        #function
    }
}

/// Generates the code for the call with the precondition handling added.
pub(crate) fn render_assert_pre(
    preconditions: PreconditionList<Precondition>,
    mut call: Call,
    attr_span: Span,
) -> Call {
    call.args_mut().push(
        parse2(quote_spanned! { attr_span=>
            ::core::marker::PhantomData::<(#preconditions)>
        })
        .expect("parses as an expression"),
    );

    call
}
