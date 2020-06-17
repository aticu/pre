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
use quote::{quote, quote_spanned, TokenStreamExt};
use std::env;
use syn::{parse2, spanned::Spanned, Ident, ItemFn, LitStr};

use crate::{
    call::Call,
    precondition::{Precondition, ReadWrite},
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

/// Renders a precondition list to a token stream.
fn render_condition_list(mut preconditions: Vec<Precondition>, span: Span) -> TokenStream {
    preconditions.sort_unstable();

    let mut tokens = TokenStream::new();
    let crate_name = get_crate_name();

    for precondition in preconditions {
        match &precondition {
            Precondition::Custom(string) => {
                tokens.append_all(quote_spanned! { precondition.span()=>
                    ::#crate_name::CustomConditionHolds::<#string>
                });
            }
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
                    ::#crate_name::ValidPtrConditionHolds::<#ident_lit, #rw_str>
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
    preconditions: Vec<Precondition>,
    mut function: ItemFn,
    span: Span,
) -> TokenStream {
    let preconditions = render_condition_list(preconditions, span);

    function.sig.inputs.push(
        parse2(quote_spanned! { span=>
            _: ::core::marker::PhantomData<(#preconditions)>
        })
        .expect("parses as a function argument"),
    );

    quote! {
        #function
    }
}

/// Generates the code for the call with the precondition handling added.
pub(crate) fn render_assert_pre(
    preconditions: Vec<Precondition>,
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
