// TODO: remove these
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, ExprCall, ItemFn};

use crate::precondition::{Precondition, PreconditionHolds, PreconditionList};

mod precondition;

cfg_if::cfg_if! {
    if #[cfg(feature = "const-generics-impl")] {
        mod const_generics_impl;
        use const_generics_impl::{render_assert_precondition, render_pre};
    } else if #[cfg(feature = "struct-impl")] {
        mod struct_impl;
        use struct_impl::{render_assert_precondition, render_pre};
    } else {
        compile_error!("you must choose one of the features providing an implementation")
    }
}

/// Allows specifing preconditions on function definitions.
///
/// This is most useful for `unsafe` functions, which are used to ["declare the existence of
/// contracts the compiler can't
/// check"](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html) for the function.
///
/// Using the `pre` macro, these contracts can be declared:
#[cfg_attr(not(feature = "const-generics-impl"), doc = "```rust")]
#[cfg_attr(feature = "const-generics-impl", doc = "```rust,ignore")]
/// # use pre_proc_macro::pre;
/// #[pre(condition("slice.len() >= 2"))]
/// unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
///     slice.get_unchecked(1)
/// }
/// ```
///
/// Callers are then forced to specify these contracts when calling the function:
#[cfg_attr(not(feature = "const-generics-impl"), doc = "```rust")]
#[cfg_attr(feature = "const-generics-impl", doc = "```rust,ignore")]
/// # #![feature(proc_macro_hygiene)]
/// # #![feature(stmt_expr_attributes)]
/// # use pre_proc_macro::{pre, assert_precondition};
/// # #[pre(condition("slice.len() >= 2"))]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// let slice = &[1, 2, 3];
/// unsafe {
///     #[assert_precondition(holds("slice.len() >= 2", reason = "slice.len() == 3"))]
///     get_second_element_unchecked(slice)
/// };
/// ```
///
/// If the contracts are not specified, compilation will fail:
#[cfg_attr(not(feature = "const-generics-impl"), doc = "```rust,compile_fail")]
#[cfg_attr(feature = "const-generics-impl", doc = "```rust,ignore")]
/// # #![feature(proc_macro_hygiene)]
/// # #![feature(stmt_expr_attributes)]
/// # use pre_proc_macro::{pre, assert_precondition};
/// # #[pre(condition("slice.len() >= 2"))]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// let slice = &[1, 2, 3];
/// unsafe {
///     get_second_element_unchecked(slice)
/// };
/// ```
///
/// If the contracts mismatch, compilation will also fail:
#[cfg_attr(not(feature = "const-generics-impl"), doc = "```rust,compile_fail")]
#[cfg_attr(feature = "const-generics-impl", doc = "```rust,ignore")]
/// # #![feature(proc_macro_hygiene)]
/// # #![feature(stmt_expr_attributes)]
/// # use pre_proc_macro::{pre, assert_precondition};
/// # #[pre(condition("slice.len() >= 2"))]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// let slice = &[1];
/// unsafe {
///     #[assert_precondition(holds("slice.len() >= 1", reason = "slice.len() == 1"))]
///     get_second_element_unchecked(slice)
/// };
/// ```
#[cfg_attr(feature = "const-generics-impl", doc = "")]
#[cfg_attr(
    feature = "const-generics-impl",
    doc = "Please note that the examples above cannot be tested when using the `const-generics-impl`"
)]
#[cfg_attr(feature = "const-generics-impl", doc = "feature.")]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn pre(attr: TokenStream, function: TokenStream) -> TokenStream {
    let preconditions = parse_macro_input!(attr as PreconditionList<Precondition>);
    let function = parse_macro_input!(function as ItemFn);

    proc_macro_error::set_dummy(quote! {
        #function
    });

    let output = render_pre(preconditions, function);

    output.into()
}

/// Assert that a precondition holds.
///
/// For more information see the documentation on the [`pre` macro](attr.pre.html)
#[proc_macro_attribute]
#[proc_macro_error]
pub fn assert_precondition(attr: TokenStream, call: TokenStream) -> TokenStream {
    let preconditions = parse_macro_input!(attr as PreconditionList<PreconditionHolds>);
    let call = parse_macro_input!(call as ExprCall);

    proc_macro_error::set_dummy(quote! {
        #call
    });

    let output = render_assert_precondition(preconditions, call);

    output.into()
}
