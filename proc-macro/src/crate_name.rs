//! Allows retrieving the name of the main crate.

use proc_macro2::Span;
use proc_macro_error::abort_call_site;
use std::env;
use syn::Ident;

/// Returns the name of the main crate.
pub(crate) fn crate_name() -> Ident {
    let name = match proc_macro_crate::crate_name("pre") {
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
