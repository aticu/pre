use proc_macro::TokenStream;
use quote::quote;
use syn::{parse2, parse_macro_input, Expr, ExprCall, ItemFn};

use crate::precondition::{Precondition, PreconditionHolds};

mod precondition;

#[proc_macro_attribute]
pub fn pre(attr: TokenStream, function: TokenStream) -> TokenStream {
    let preconditions = parse_macro_input!(attr as Precondition);
    let mut function = parse_macro_input!(function as ItemFn);

    let function_name = function.sig.ident.clone();
    let precondition_rendered = preconditions.render_as_ident();

    let struct_def = quote! {
        #[allow(non_camel_case_types)]
        struct #function_name {
            #precondition_rendered: ()
        }
    };

    function.sig.inputs.push(
        parse2(quote! {
            _: #function_name
        })
        .unwrap(),
    );

    let output: TokenStream = quote! {
        #struct_def
        #function
    }
    .into();

    output
}

#[proc_macro_attribute]
pub fn assert_precondition(attr: TokenStream, call: TokenStream) -> TokenStream {
    let preconditions = parse_macro_input!(attr as PreconditionHolds);
    let mut call = parse_macro_input!(call as ExprCall);
    let path;

    if let Expr::Path(p) = *call.func.clone() {
        path = p;
    } else {
        panic!("unable to exactly determine at compile time which function is being called");
    }
    let precondition_rendered = preconditions.render_as_ident();

    call.args.push(
        parse2(quote! {
            #path {
                #precondition_rendered: ()
            }
        })
        .unwrap(),
    );

    let output: TokenStream = quote! {
        #call
    }
    .into();

    output
}
