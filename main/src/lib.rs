//! `pre` is a library to help programmers correctly uphold preconditions for function calls.
//!
//! # Motivation
//!
//! Sometimes functions or methods have preconditions that cannot be ensured in the type system and
//! cannot be guarded against at runtime.
//! The most prominent example of functions like that are `unsafe` functions.
//! When used correctly, `unsafe` functions are used to ["declare the existence of
//! contracts the compiler can't
//! check"](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html).
//! These contracts are the preconditions for the function call.
//! Failing to uphold them usually results in a violation of memory safety and undefined behavior.
//!
//! Currently the most used scheme for dealing with these preconditions on `unsafe` functions is to
//! mention them in the `Safety` section of the function's documentation. Programmers using the
//! function then have to check what they have to ensure to call the function correctly. The
//! programmer that uses the function may then leave a comment next to the function, describing why
//! the call is safe (why the preconditions hold).
//!
//! This approach is even advertised by the compiler (as of 1.44.1) when using an `unsafe` function
//! outside of an `unsafe` block:
//!
//! ```text
//! note: consult the function's documentation for information on how to avoid undefined behavior
//! ```
//!
//! There are multiple points of failure with this approach. Among others these are:
//!
//! 1. The original function could not document what the preconditions are.
//! 2. The programmer using the function could forget to look at the documented preconditions.
//! 3. The programmer using the function could overlook one or more of the preconditions. This is
//!    not unlikely, if multiple preconditions are documented in a single long paragraph.
//! 4. The programmer could make a mistake when checking whether the preconditions hold.
//! 5. An update could change the preconditions that the function requires and users of the
//!    function could easily miss that fact.
//! 6. The programmer could forget (or choose not to) document why the preconditions hold when
//!    calling the function, making it easier for others to make a mistake when later changing the
//!    call or code around it.
//!
//! This library cannot guard against all of these problems, especially not against 1 and 4.
//! It however attempts to help against 2, 3, 5 and 6.
//!
//! # The approach
//!
//! This library works by allowing programmers to specify preconditions on functions they write in
//! a unified format.
//! Those preconditions are then transformed into an additional function argument.
//! Callers of the function then specify the same preconditions at the call site, along with a
//! reason why they believe the precondition is upheld.
//! If the preconditions don't match or are not specified, the function will have invalid arguments
//! and the code will not compile. This should protect against problems 2, 3 and 5 from above.
//! Because a reason is required at the function call site, problem 6 is at least partially guarded
//! against, though programmers may still choose to not put too much effort into the reason.
//!
//! The signature of the functions is changed by adding a single *zero-sized* parameter.
//! **This means that when compiled using the release mode, there is no run-time cost for these
//! checks. They are a zero-cost abstraction.**
//!
//! # Usage
//!
//! The basic usage for the `pre` crate looks like this:
//!
//! ```rust
//! use pre::pre;
//!
//! #[pre("`arg` must have a meaningful value")]
//! fn foo(arg: i32) {
//!     assert_eq!(arg, 42);
//! }
//!
//! #[pre] // Enables `assure`ing preconditions inside the function
//! fn main() {
//!     #[assure("`arg` must have a meaningful value", reason = "42 is very meaningful")]
//!     foo(42);
//! }
//! ```
//!
//! The [`pre` attribute](attr.pre.html) serves to specify preconditions (on `foo`) and to enable
//! usage of the `assure` attribute (on `main`). To learn why the second usage is necessary, read
//! the [paragraph about the checking functionality](attr.pre.html#checking-functionality) on the
//! documentation of the `pre` attribute.
//!
//! With the [`assure` attribute](attr.assure.html) the programmer assures that the precondition
//! was checked by them and is upheld.
//! Without the `assure` attribute, the code would fail to compile.
//!
//! **The precondition inside the `assure` attribute must be exactly equal to the precondition
//! inside the `pre` attribute at the function definition for the code to compile.**
//! The order of the preconditions, if there are multiple, does not matter however.
//!
//! ```rust,compile_fail
//! use pre::pre;
//!
//! #[pre("`arg` must have a meaningful value")]
//! fn foo(arg: i32) {
//!     assert_eq!(arg, 42);
//! }
//!
//! fn main() {
//!     foo(42);
//! }
//! ```

#![allow(clippy::needless_doctest_main)]
#![cfg_attr(nightly, feature(const_generics))]
#![cfg_attr(nightly, allow(incomplete_features))]

/// Specify preconditions on functions and check that they are assured correctly for calls.
///
/// Its counterpart is the [`assure` attribute](attr.assure.html).
///
/// # General syntax
///
/// There are three uses of the `pre` attribute:
///
/// 1. Specify one or multiple preconditions (for the exact syntax of the preconditions, see
///    ["Precondition syntax"](#precondition-syntax)):
///
///    ```rust,ignore
///    #[pre(<first precondition>)]
///    #[pre(<second precondition>)]
///    #[pre(<third precondition>)]
///    fn foo() {}
///    ```
/// 2. Enable handling of [`assure`](attr.assure.html) and [`forward`](attr.forward.html)
///    attributes for the annotated item (see ["Checking functionality"](#checking-functionality)):
///
///    ```rust
///    # use pre::pre;
///    #
///    #[pre]
///    fn main() {
///        // `assure` and `forward` attributes here are properly handled.
///    }
///    ```
///
///    This works with any of the forms of the `pre` attribute, but in case no other functionality
///    is needed, a bare `#[pre]` can be used.
/// 3. Disable generating documentation for the preconditions (see ["Documentation on items with
///    preconditions"](#documentation-on-items-with-preconditions)):
///
///    ```rust
///    # use pre::pre;
///    #
///    #[pre(no_doc)]
///    #[pre("some precondition")]
///    fn foo() {} // foo will not have any documentation generated by `pre`.
///    ```
///
/// # Precondition syntax
///
/// There are multiple different types of preconditions that you can use for your functions:
///
/// 1. Custom preconditions:
///
///    This is the simplest kind of precondition. It is a string describing the condition.
///
///    The syntax is `#[pre("<string>")]`.
///
///    - `<string>`: An arbitrary string describing the condition.
///
///    ### Example
///
///    ```rust
///    # use pre::pre;
///    #
///    #[pre("describe your precondition here")]
///    fn foo() {}
///    ```
/// 2. Valid pointer preconditions:
///
///    This precondition requires that a raw pointer is
///    [valid](https://doc.rust-lang.org/std/ptr/index.html#safety) for reads or writes or both.
///
///    The syntax is `#[pre(valid_ptr(<ptr_name>, <access_modes>))]`.
///
///    - `<ptr_name>`: The identifier of the pointer argument that must be valid.
///    - `<access_modes>`: One of `r`, `w` or `r+w`. This specifies whether the pointer is valid
///    for reads (`r`) or writes (`w`) or both (`r+w`).
///
///    ### Example
///
///    ```rust
///    # use pre::pre;
///    #
///    #[pre(valid_ptr(ptr_name, r+w))]
///    fn foo(ptr_name: *mut i32) {}
///    ```
///
///    This precondition **does not** guarantee:
///
///    - A valid alignment of the pointer.
///    - A valid initialized value for the pointee.
///
///    Also there are no guarantees about the size of the allocated object.
///    If there are no other preconditions about the size of the allocated object, usually the size
///    of a single value can be assumed.
///
/// # Checking functionality
///
/// The `pre` attribute can also be used to enable the functionalities of the
/// [`assure`](attr.assure.html) and [`forward`](attr.forward.html) attributes for the item it is
/// attached to. In this case it can be left empty: `#[pre]`. Any other form of a `pre` attribute
/// being present on an item renders the empty one obsolete though, as any of its forms enables
/// this functionality.
///
/// Doing this is currently necessary, because the current (1.44.1) stable rust compiler does not
/// support attribute macros being applied to statements or expressions directly.
///
/// # Documentation on items with preconditions
///
/// Items annotated with one or more preconditions have information about their preconditions
/// and how to call them with the preconditions appended at the end of their documentation.
///
/// If you wish not to add such documentation to a particular item, you can add `#[pre(no_doc)]` to
/// the attributes of the item, to prevent its generation.
pub use pre_proc_macro::pre;

/// Assure that a precondition holds.
///
/// This is the counterpart of the [`pre` attribute](attr.pre.html).
///
/// Currently this attribute does not work by itself.
/// It needs to be used inside of a context that is annotated by a `pre` attribute.
///
/// # Terminology
///
/// The term `assure` was chosen, because it most accurately describes the function of the
/// attribute.
///
/// There are no guarantees that a precondition holds, other than the fact that the programmer
/// promises that it does. The burden is still on the programmer in this case. `pre` only makes
/// sure that the programmer cannot forget that a precondition exists and that no precondition is
/// changed without the programmer noticing.
///
/// # Syntax
///
/// The basic syntax of the `assure` attribute is:
///
/// ```rust,ignore
/// #[assure(<first precondition>, reason = "<the reason why the first precondition can be assured>")]
/// #[assure(<second precondition>, reason = "<the reason why the second precondition can be assured>")]
/// #[assure(<third precondition>, reason = "<the reason why the third precondition can be assured>")]
/// foo();
/// ```
///
/// To learn more about the precondition syntax and the possible types of preconditions, you should
/// look at the [documentation of the `pre` attribute](attr.pre.html#precondition-syntax).
pub use pre_proc_macro::assure;

/// Forward the call to a different function that has the preconditions for the original function.
///
/// Currently this attribute does not work by itself.
/// It needs to be used inside of a context that is annotated by a `pre` attribute.
///
/// The purpose of this attribute is to apply the preconditions defined on items in a
/// [`extern_crate` module](attr.extern_crate.html) to the use of the original item.
///
/// It is useful if you want to check preconditions for an existing file without making
/// modifications to the file besides adding attributes.
///
/// For most use cases, it is better to just use the item inside of the `extern_crate` module
/// directly. The only case where this attribute is currently absolutely needed, is if you want to
/// check preconditions for functions and methods defined inside of an `impl` block in an
/// `extern_crate` attribute.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use pre::pre;
///
/// #[pre::extern_crate(std)]
/// mod new_std {
///     mod ptr {
///         #[pre(valid_ptr(dst, w))]
///         unsafe fn write_unaligned<T>(dst: *mut T, src: T);
///     }
/// }
///
/// #[pre]
/// fn main() {
///     // Replacing this with `new_std::ptr::write_unaligned` would make the first `forward`
///     // attribute below unnecessary.
///     use std::ptr::write_unaligned;
///
///     let mut x = 0;
///
///     // No preconditions are checked here.
///     unsafe { write_unaligned(&mut x, 1) };
/// #   assert_eq!(x, 1);
///
///     // Here the actual function being called is `new_std::ptr::write_unaligned`, which checks
///     // the preconditions and is otherwise functionally equivalent to
///     // `std::ptr::write_unaligned`.
///     #[forward(new_std::ptr)]
///     #[assure(valid_ptr(dst, w), reason = "`dst` is created from a reference")]
///     unsafe { write_unaligned(&mut x, 2) };
/// #   assert_eq!(x, 2);
///
///     // Here the `std` segment of the path is replaced with `new_std`, so again
///     // `new_std::ptr::write_unaligned` is called.
///     // The same effect could be achieved without a `forward` attribute, by replacing the `std`
///     // in the path of the call with `new_std`.
///     #[forward(std -> new_std)]
///     #[assure(valid_ptr(dst, w), reason = "`dst` is created from a reference")]
///     unsafe { std::ptr::write_unaligned(&mut x, 3) };
/// #   assert_eq!(x, 3);
/// }
/// ```
///
/// For functions and methods inside of `impl` blocks, using the `forward` attribute with the `impl`
/// keyword inside is necessary to check the preconditions:
///
/// ```rust
/// use pre::pre;
///
/// #[pre::extern_crate(std)]
/// mod new_std {
///     mod ptr {
///         impl<T> NonNull<T> {
///             #[pre("`ptr` must be non-null")]
///             const unsafe fn new_unchecked(ptr: *mut T) -> NonNull<T>;
///         }
///     }
/// }
///
/// #[pre]
/// fn main() {
///     let mut val = 0;
///
///     // Even though this uses the `NonNull` type through the `extern_crate` module, this will
///     // unfortunately not check the preconditions.
///     let non_null = unsafe { new_std::ptr::NonNull::new_unchecked(&mut val) };
///
///     // This call actually checks the preconditions. Note the `impl` keyword before the path
///     // below. This is required, so that the preconditions can be properly checked for a
///     // function in an `impl` block inside of a `extern_crate` module.
///     #[forward(impl new_std::ptr::NonNull)]
///     #[assure("`ptr` must be non-null", reason = "a reference is never null")]
///     let non_null = unsafe { new_std::ptr::NonNull::new_unchecked(&mut val) };
///
///     // The same thing also works when using the `NonNull` through the `std::ptr` path.
///     #[forward(impl new_std::ptr::NonNull)]
///     #[assure("`ptr` must be non-null", reason = "a reference is never null")]
///     let non_null = unsafe { std::ptr::NonNull::new_unchecked(&mut val) };
/// }
/// ```
///
/// # Syntax
///
/// This attribute has three different forms:
///
/// - [Direct call](#direct-call)
/// - [Path replacement](#path-replacement)
/// - [Impl call](#impl-call)
///
/// ## Direct call
///
/// `#[forward(<path>)]`
///
/// ### How it works
///
/// `<path>` is prepended to the path of the annotated call.
///
/// ### Example
///
/// ```rust,ignore
/// #[forward(abc::def)]
/// ghi::jkl();
/// ```
///
/// becomes
///
/// ```rust,ignore
/// abc::def::ghi::jkl();
/// ```
///
/// ## Path replacement
///
/// `#[forward(<old_path> -> <new_path>)]`
///
/// ### How it works
///
/// `<old_path>` is replaced with `<new_path>` in the path of the annotated call.
///
/// ### Example
///
/// ```rust,ignore
/// #[forward(abc -> def)]
/// abc::ghi::jkl();
/// ```
///
/// becomes
///
/// ```rust,ignore
/// def::ghi::jkl();
/// ```
///
/// ## Impl call
///
/// `#[forward(impl <path>)]`
///
/// ### How it works
///
/// Instead of checking the preconditions on the original method call, the preconditions are
/// checked for a function or method with the same name located at an `impl` block at `<path>`.
/// For this to work, the `impl` block at `<path>` must be inside of an
/// [`extern_crate`](attr.extern_crate.html)-annotated module.
///
/// ### Example
///
/// ```rust,ignore
/// let v: SomeType<bool> = SomeType::new();
///
/// #[forward(impl some_impl::SomeType)]
/// #[pre(<some_condition>)]
/// v.some_method(some_arg);
/// ```
///
/// works similar to
///
/// ```rust,ignore
/// let v: SomeType<bool> = SomeType::new();
///
/// #[pre(<some_condition>)]
/// some_impl::SomeType::some_method(); // Does not actually do anything, just checks the
///                                     // preconditions.
/// v.some_method(some_arg);
/// ```
///
/// The exact inner workings of this are different to make it work in more contexts, but this is a
/// good mental model to think about it.
pub use pre_proc_macro::forward;

/// Provide preconditions for items in a different crate.
///
/// This attribute can be used when a library has documented preconditions without using `pre` and
/// you want those preconditions to be checked by `pre`.
///
/// It works by specifying an outline of the library as a module.
/// Every function that should have preconditions added is simply referenced by it's signature.
///
/// The module then acts as a drop-in replacement for the original library.
///
/// # Example
///
/// ```rust
/// use pre::pre;
///
/// #[pre::extern_crate(core)]
/// mod new_core {
///     mod mem {
///         // Notice that the body of the function is missing.
///         // It's behavior is exactly the same as `core::mem::zeroed`.
///         #[pre("an all-zero byte-pattern must be valid for `T`")]
///         unsafe fn zeroed<T>() -> T;
///
///         impl<T> MaybeUninit<T> {
///             #[pre("the contained value must be an initialized, valid value of `T`")]
///             unsafe fn assume_init(self) -> T;
///         }
///     }
/// }
///
/// #[pre]
/// fn main() {
///     // Note the use of `new_core` instead of `core`.
///     use new_core::mem;
///
///     // When using functions from `new_core`, the preconditions applied there are checked, but
///     // the behavior is the same as the original function in `core`.
///     #[assure(
///         "an all-zero byte-pattern must be valid for `T`",
///         reason = "`usize` supports an all-zero byte-pattern"
///     )]
///     let x: usize = unsafe { mem::zeroed() };
///     assert_eq!(x, 0);
///
///     // Functions and types available in the original library (`core` in this case) are
///     // accessible unaltered through the new name `new_core`.
///     let mut b = new_core::mem::MaybeUninit::uninit();
///
///     // No preconditions were specified for `new_core::ptr::write` in this example. In a
///     // realistic setting, this should be done. Here is just serves as an example that the
///     // function is available in `new_core` without being explicitly mentioned.
///     unsafe { new_core::ptr::write(b.as_mut_ptr(), true) };
///
///     // The `forward` attribute here is required to find the preconditions for this function.
///     #[forward(impl new_core::mem::MaybeUninit)]
///     #[assure(
///         "the contained value must be an initialized, valid value of `T`",
///         reason = "the value `true` was just written to `b`"
///     )]
///     let val = unsafe { b.assume_init() };
///     assert_eq!(val, true);
/// }
/// ```
///
/// Note the use of the [`forward` attribute](attr.forward.html) above. For more information about
/// it and its use, you can read [its documentation](attr.forward.html).
///
/// # Visibility
///
/// Visibility modifiers on inner items of the module are ignored.
///
/// Instead it is ensured that inner modules are always visible in any place the topmost module is
/// visible.
/// You can think of every item in the contained module having `pub` visibility (though in practice
/// it's slightly more complicated).
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
