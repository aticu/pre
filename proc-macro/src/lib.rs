use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{emit_warning, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, Item, ItemFn};

use crate::{
    assert_pre::AssertPreVisitor,
    precondition::{Precondition, PreconditionList},
};

mod assert_pre;
mod precondition;

cfg_if::cfg_if! {
    if #[cfg(feature = "const-generics-impl")] {
        mod const_generics_impl;
        pub(crate) use crate::const_generics_impl::{render_assert_pre, render_pre};
    } else if #[cfg(feature = "struct-impl")] {
        mod struct_impl;
        pub(crate) use crate::struct_impl::{render_assert_pre, render_pre};
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
///     #[assert_pre(condition("slice.len() >= 2", reason = "slice.len() == 3"))]
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
///     #[assert_pre(condition("slice.len() >= 1", reason = "slice.len() == 1"))]
///     get_second_element_unchecked(slice)
/// };
/// ```
#[cfg_attr(feature = "const-generics-impl", doc = "")]
#[cfg_attr(
    feature = "const-generics-impl",
    doc = "Please note that the examples above cannot be tested when using the `const-generics-impl`"
)]
#[cfg_attr(feature = "const-generics-impl", doc = "feature.")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn pre(attr: TokenStream, function: TokenStream) -> TokenStream {
    let dummy_function: TokenStream2 = function.clone().into();
    proc_macro_error::set_dummy(quote! {
        #dummy_function
    });

    let preconditions = parse_macro_input!(attr as PreconditionList<Precondition>);
    let function = parse_macro_input!(function as ItemFn);

    let output = render_pre(preconditions, function);

    // Reset the dummy here, in case errors were emitted in `render_pre`.
    // This will use the most up-to-date version of the function.
    proc_macro_error::set_dummy(quote! {
        #dummy_function
    });

    output.into()
}

/// Check that the `assert_pre` attribute is applied correctly in the enclosing scope.
///
/// This is required, because with the current stable rust compiler, attribute macros cannot be
/// applied to statements or expressions directly.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn check_pre(attr: TokenStream, item: TokenStream) -> TokenStream {
    let dummy_item: TokenStream2 = item.clone().into();
    proc_macro_error::set_dummy(quote! {
        #dummy_item
    });

    if !attr.is_empty() {
        let attr: TokenStream2 = attr.into();
        emit_warning!(attr, "this does not do anything and is ignored");
    }

    let mut item = parse_macro_input!(item as Item);

    AssertPreVisitor.visit_item_mut(&mut item);

    let output = quote! {
        #item
    };

    // Reset the dummy here, in case errors were emitted in visiting the syntax tree.
    // This will use the most up-to-date version of the function.
    proc_macro_error::set_dummy(quote! {
        #output
    });

    output.into()
}
