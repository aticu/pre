#![allow(clippy::needless_doctest_main)]
#![cfg_attr(nightly, feature(const_generics))]
#![cfg_attr(nightly, allow(incomplete_features))]

/// Allows specifing preconditions on function definitions.
///
/// This is most useful for `unsafe` functions, which are used to ["declare the existence of
/// contracts the compiler can't
/// check"](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html) for the function.
///
/// Using the `pre` macro, these contracts can be declared in the form of preconditions:
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
/// Callers are then forced to specify these preconditions when calling the function:
///
/// ```rust
/// # use pre::pre;
/// #
/// # #[pre("slice.len() >= 2")]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// #[pre]
/// fn main() {
///     let slice = &[1, 2, 3];
///     unsafe {
///         #[assure("slice.len() >= 2", reason = "slice.len() == 3")]
///         get_second_element_unchecked(slice)
///     };
/// }
/// ```
///
/// Notice the use of the `pre` attributeon the main function. This is required to call a function
/// with specified preconditions directly. If you want more information on why it is required, look
/// at the ["Checking functionality" section](#checking-functionality).
///
/// If the preconditions are not specified, compilation will fail:
///
/// ```rust,compile_fail
/// # use pre::pre;
/// #
/// # #[pre("slice.len() >= 2")]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// #[pre]
/// fn main() {
///     let slice = &[1, 2, 3];
///     unsafe {
///         get_second_element_unchecked(slice)
///     };
/// }
/// ```
///
/// If the preconditions mismatch, compilation will also fail:
///
/// ```rust,compile_fail
/// # use pre::pre;
/// #
/// # #[pre("slice.len() >= 2")]
/// # unsafe fn get_second_element_unchecked(slice: &[i32]) -> &i32 {
/// #     slice.get_unchecked(1)
/// # }
/// #
/// #[pre]
/// fn main() {
///     let slice = &[1];
///     unsafe {
///     #[assure("slice.len() >= 1", reason = "slice.len() == 1")]
///         get_second_element_unchecked(slice)
///     };
/// }
/// ```
///
/// # Checking functionality
///
/// The `pre` attribute can also be used to enable checking of preconditions for the item it is
/// attached to. In this case it can be left empty. For an example look at the `main` function
/// above.
///
/// Doing this is currently necessary, because the current stable rust compiler does not support
/// attribute macros being applied to statements or expressions directly.
pub use pre_proc_macro::pre;

/// Assure that a precondition holds.
pub use pre_proc_macro::assure;

/// Provide preconditions for items in a different crate.
pub use pre_proc_macro::extern_crate;

cfg_if::cfg_if! {
    if #[cfg(nightly)] {
        // *WARNING* These types are not considered to be part of the public API and may change at
        // any time without notice.

        /// A declaration that a custom condition holds.
        #[doc(hidden)]
        pub struct CustomConditionHolds<const CONDITION: &'static str>;

        /// A declaration that the pointer of name `PTR` is valid.
        #[doc(hidden)]
        pub struct ValidPtrConditionHolds<const PTR: &'static str, const ACCESS_TYPE: &'static str>;
    }
}
