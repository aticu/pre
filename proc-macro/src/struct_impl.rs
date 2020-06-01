//! Implements the procedural macros using generated structs passed as an additional parameter.
//!
//! # Advantages of this approach
//! - uses only stable features
//! - quick to compute
//!
//! # Disadvantages of this approach
//! - possible name clashes, because the identifier namespace is limited
//! - error messages not very readable
//! - the struct must be defined somewhere, which is not possible for a method

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Expr, ExprCall, ItemFn};

use crate::precondition::{Precondition, PreconditionHolds};

/// Renders a precondition as a `String` representing an identifier.
pub(crate) fn render_as_ident(precondition: &PreconditionKind) -> Ident {
    /// Escapes characters that are not valid in identifiers.
    fn escape_non_ident_chars(string: String) -> String {
        string
            .chars()
            .map(|c| match c {
                '_' | '0'..='9' | 'a'..='z' | 'A'..='Z' => c.to_string(),
                other => format!("_{:x}", other as u32),
            })
            .collect()
    }

    match precondition {
        PreconditionKind::ValidPtr { ident, .. } => format_ident!("_valid_ptr_{}", ident),
        PreconditionKind::Custom(string) => {
            format_ident!("_custom_{}", escape_non_ident_chars(string.value()))
        }
    }
}

/// Generates the code for the function with the precondition handling added.
pub(crate) fn render_pre(preconditions: Precondition, mut function: ItemFn) -> TokenStream {
    let function_name = function.sig.ident.clone();
    let precondition_rendered = render_as_ident(preconditions.kind());

    let struct_def = quote! {
        #[allow(non_camel_case_types)]
        struct #function_name {
            #precondition_rendered: ()
        }
    };

    function.sig.inputs.push(parse_quote! {
        _: #function_name
    });

    quote! {
        #struct_def
        #function
    }
}

/// Generates the code for the call with the precondition handling added.
pub(crate) fn render_assert_precondition(
    preconditions: PreconditionHolds,
    mut call: ExprCall,
) -> TokenStream {
    let path;

    if let Expr::Path(p) = *call.func.clone() {
        path = p;
    } else {
        panic!("unable to exactly determine at compile time which function is being called");
    }
    let precondition_rendered = render_as_ident(preconditions.kind());

    call.args.push(parse_quote! {
        #path {
            #precondition_rendered: ()
        }
    });

    quote! {
        #call
    }
}
