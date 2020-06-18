#![allow(clippy::needless_doctest_main)]
#![cfg_attr(nightly, feature(const_generics))]
#![cfg_attr(nightly, allow(incomplete_features))]

/// Allows specifing preconditions on function definitions.
///
/// This is most useful for `unsafe` functions, which are used to ["declare the existence of
/// contracts the compiler can't
/// check"](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html) for the function.
///
/// Using the `pre` macro, these contracts can be declared:
///
/// ```rust
/// use pre::pre;
///
/// #[pre("slice.len() >= 2")]
/// unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
///     slice.get_unchecked(1)
/// }
/// ```
///
/// Callers are then forced to specify these contracts when calling the function:
///
/// ```rust
/// # use pre::pre;
/// #
/// # #[pre("slice.len() >= 2")]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// use pre::check_pre;
///
/// #[check_pre]
/// fn main() {
///     let slice = &[1, 2, 3];
///     unsafe {
///         #[assert_pre("slice.len() >= 2", reason = "slice.len() == 3")]
///         get_second_element_unchecked(slice)
///     };
/// }
/// ```
///
/// Notice the use of the [`check_pre` attribute](attr.check_pre.html) on the main function. This
/// is required to call a function with specified preconditions directly. If you want more
/// information on why it is required, look at [its documentation](attr.check_pre.html).
///
/// If the contracts are not specified, compilation will fail:
///
/// ```rust,compile_fail
/// # use pre::pre;
/// #
/// # #[pre("slice.len() >= 2")]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// use pre::check_pre;
///
/// #[check_pre]
/// fn main() {
///     let slice = &[1, 2, 3];
///     unsafe {
///         get_second_element_unchecked(slice)
///     };
/// }
/// ```
///
/// If the contracts mismatch, compilation will also fail:
///
/// ```rust,compile_fail
/// # use pre::pre;
/// #
/// # #[pre("slice.len() >= 2")]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// use pre::check_pre;
///
/// #[check_pre]
/// fn main() {
///     let slice = &[1];
///     unsafe {
///     #[assert_pre("slice.len() >= 1", reason = "slice.len() == 1")]
///         get_second_element_unchecked(slice)
///     };
/// }
/// ```
pub use pre_proc_macro::pre;

/// Check that the `assert_pre` attribute is applied correctly in the enclosing scope.
///
/// For more information, look at the documentation of the [`pre` attribute](attr.pre.html).
///
/// Using this attribute is currently necessary, because the current stable rust compiler does not
/// support attribute macros being applied to statements or expressions directly.
///
/// Basic usage looks like this:
///
/// ```rust
/// use pre::check_pre;
///
/// #[check_pre]
/// fn main() {
///     /* calls to functions with preconditions here */
/// }
/// ```
pub use pre_proc_macro::check_pre;

/// Provide preconditions for items in a different crate.
pub use pre_proc_macro::pre_defs_for;

#[cfg(nightly)]
mod const_generics_types;

#[cfg(nightly)]
#[doc(hidden)]
pub use const_generics_types::*;
