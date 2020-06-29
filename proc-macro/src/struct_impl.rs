//! Implements the procedural macros using generated structs passed as an additional parameter.
//!
//! The struct has the same name as the function to avoid having to know how to import it.
//! See [this
//! description](https://github.com/dtolnay/case-studies/tree/master/unit-type-parameters/README.md)
//! if you want to understand how it works. It describes solving the import issue for a different
//! problem.
//!
//! # Advantages of this approach
//! - uses only stable features
//! - quick to compute
//!
//! # Disadvantages of this approach
//! - possible name clashes, because the identifier namespace is limited
//! - error messages not very readable
//! - the struct must be defined somewhere, which is not possible for a method
//!
//! # What the generated code looks like
//!
//! ```rust,ignore
//! #[pre::pre("some_val > 42.0")]
//! fn has_preconditions(some_val: f32) -> f32 {
//!     assert!(some_val > 42.0);
//!
//!     some_val
//! }
//!
//! #[pre::pre]
//! fn main() {
//!     #[assure("some_val > 42.0", reason = "43.0 > 42.0")]
//!     has_preconditions(43.0);
//! }
//! ```
//!
//! turns into
//!
//! ```rust,ignore
//! #[allow(non_camel_case_types)]
//! #[allow(non_snake_case)]
//! struct has_preconditions {
//!     _custom_some__val_20_3e_2042_2e0: (),
//! }
//!
//! fn has_preconditions(some_val: f32, _: has_preconditions) -> f32 {
//!     assert!(some_val > 42.0);
//!
//!     some_val
//! }
//!
//! fn main() {
//!     has_preconditions(
//!         43.0,
//!         has_preconditions {
//!             _custom_some__val_20_3e_2042_2e0: (),
//!         },
//!     );
//! }
//! ```

use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{format_ident, quote, quote_spanned, TokenStreamExt};
use syn::{parse2, spanned::Spanned, Ident, ItemFn};

use crate::{
    call::Call,
    precondition::{Precondition, ReadWrite},
};

/// Renders a precondition as a `String` representing an identifier.
pub(crate) fn render_as_ident(precondition: &Precondition) -> Ident {
    /// Escapes characters that are not valid in identifiers.
    fn escape_non_ident_chars(string: String) -> String {
        string
            .chars()
            .map(|c| match c {
                '0'..='9' | 'a'..='z' | 'A'..='Z' => c.to_string(),
                '_' => "__".to_string(), // escape `'_'` to prevent name clashes
                other => format!("_{:x}", other as u32),
            })
            .collect()
    }

    let mut ident = match precondition {
        Precondition::ValidPtr {
            ident, read_write, ..
        } => format_ident!(
            "_valid_ptr_{}_{}",
            ident,
            match read_write {
                ReadWrite::Read { .. } => "r",
                ReadWrite::Write { .. } => "w",
                ReadWrite::Both { .. } => "rw",
            }
        ),
        Precondition::Custom(string) => {
            format_ident!("_custom_{}", escape_non_ident_chars(string.value()))
        }
    };

    ident.set_span(precondition.span());

    ident
}

/// Generates the code for the function with the precondition handling added.
pub(crate) fn render_pre(
    preconditions: Vec<Precondition>,
    function: &mut ItemFn,
    span: Span,
) -> TokenStream {
    if function.sig.receiver().is_some() {
        emit_error!(
            span,
            "preconditions are not supported for methods on the stable compiler"
        );
        return quote! { #function };
    }

    let vis = &function.vis;
    let mut preconditions_rendered = TokenStream::new();
    preconditions_rendered.append_all(
        preconditions
            .iter()
            .map(render_as_ident)
            .map(|ident| quote_spanned! { span=> #vis #ident: (), }),
    );

    let function_name = function.sig.ident.clone();
    let struct_def = quote_spanned! { span=>
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[cfg(not(doc))]
        #vis struct #function_name {
            #preconditions_rendered
        }
    };

    function.sig.inputs.push(
        parse2(quote_spanned! { span=>
            #[cfg(not(doc))]
            _: #function_name
        })
        .expect("parses as valid function argument"),
    );

    quote! {
        #struct_def
        #function
    }
}

/// Generates the code for the call with the precondition handling added.
pub(crate) fn render_assure(preconditions: Vec<Precondition>, mut call: Call, span: Span) -> Call {
    if !call.is_function() {
        emit_error!(
            call,
            "method calls are not supported by `pre` on the stable compiler"
        );

        return call;
    }

    let path;

    if let Some(p) = call.path() {
        path = p;
    } else {
        match &call {
            Call::Function(call) => emit_error!(
                call,
                "unable to determine at compile time which function is being called";
                help = "use a direct path to the function instead"
            ),
            _ => unreachable!("we already checked that it's a function"),
        }

        return call;
    }

    let mut preconditions_rendered = TokenStream::new();
    preconditions_rendered.append_all(
        preconditions
            .iter()
            .map(render_as_ident)
            .map(|ident| quote_spanned! { span=> #ident: (), }),
    );

    call.args_mut().push(
        parse2(quote_spanned! { span=>
            #path {
                #preconditions_rendered
            }
        })
        .expect("parses as an expression"),
    );

    call
}
