//! pre is a library to help programmers correctly uphold preconditions for function calls.
//!
//! # Motivation
//!
//! Sometimes functions or methods have preconditions that cannot be ensured in the type system and
//! cannot be guarded against at runtime.
//! The most prominent example of functions like that are `unsafe` functions.
//! When used correctly, `unsafe` functions are used to ["declare the existence of contracts the
//! compiler can't check"](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html).
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
//! against, though programmers could still choose to not put too much effort into the reason.
//!
//! The signature of the functions is changed by adding a single *zero-sized* parameter.
//! **This means that when compiled using the release mode, there is no run-time cost for these
//! checks. They are a zero-cost abstraction.**
//!
//! # Usage
//!
//! The basic usage for the pre crate looks like this:
//!
//! ```rust
//! use pre::pre;
//!
//! #[pre("`arg` is a meaningful value")]
//! fn foo(arg: i32) {
//!     assert_eq!(arg, 42);
//! }
//!
//! #[pre] // Enables `assure`ing preconditions inside the function
//! fn main() {
//!     #[assure("`arg` is a meaningful value", reason = "42 is very meaningful")]
//!     foo(42);
//! }
//! ```
//!
//! The [`pre` attribute] serves to specify preconditions (on `foo`) and to enable usage of the
//! `assure` attribute (on `main`).  To learn why the second usage is necessary, read the paragraph
//! about the [checking functionality] on the documentation of the `pre` attribute.
//!
//! With the [`assure` attribute] the programmer assures that the precondition was checked by them
//! and is upheld. Without the `assure` attribute, the code would fail to compile.
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
//! //  ^^^^^^^-- this errors
//! }
//! ```
//!
//! **The precondition inside the `assure` attribute must be exactly equal to the precondition
//! inside the `pre` attribute at the function definition for the code to compile.**
//! The order of the preconditions, if there are multiple, does not matter however.
//!
//! # Known Limitations
//!
//! There are many subtleties involved when working with `unsafe` code. pre is supposed to help
//! programmers know where to look, but it does not do anything beyond that. The programmer still
//! has to manually check all the contracts of the `unsafe` code. Therefore even when using pre
//! you should still **always check the "Safety" section of the documentation**.
//!
//! There are also some technical limitations to what pre can do:
//!
//! - There is more than one form of `unsafe` code. pre currently exclusively focuses on `unsafe`
//!   functions.
//! - While pre does work on the stable compiler, there are quite a few things that only work
//!   when using the nightly compiler.
//!
//!   These are the main differences between the nightly version and the stable version (there are
//!   other minor ones):
//!     - **Preconditions on functions in `impl` blocks only work on nightly.**
//!
//!       This does not apply to `impl` blocks inside of an `extern_crate` annotated module. These
//!       have their own limitations though (see below).
//!     - Warnings from pre are only possible on nightly.
//!     - Errors can reference multiple locations providing better suggestions and messages on
//!       nightly.
//! - Since pre works by adding an additional argument to a function, it changes the function
//!   signature. That won't make a difference in many cases, but if you use function pointers or
//!   pass a function as an argument, it will have a different type from what it appears to be.
//! - Because attribute macros are not supported for expressions and statements on the current
//!   stable compiler, functions that contain an `assure` attribute must have at least one `pre`
//!   attribute, though it could be empty: [`#[pre]`][checking functionality].
//! - pre was designed with the 2018 edition in mind. While it does work with the 2015 edition, it
//!   may be necessary to add an `extern crate core` statement, if you don't have one yet. Also the
//!   [`extern_crate` attribute] is not supported with the 2015 edition.
//! - While using any of pre's attributes within a [`cfg_attr` attribute] works, there are two
//!   limitations to that:
//!     - All `cfg_attr` attributes must have the same configuration predicates. The same here
//!       means syntactic equality, so `all(unix, target_endian = "little")` is not the same as
//!       `all(target_endian = "little", unix)`. This is done easiest, by simply putting all
//!       preconditions behind a single `cfg_attr`.
//!     - Nested `cfg_attr` attributes are not supported, so `#[cfg_attr(unix,
//!       cfg_attr(target_endian = "little", assure(...)))]` is currently not recognized by pre.
//! - There are multiple limitations for functions and methods defined in a module which is
//!   annotated with the [`extern_crate` attribute] or has a parent that is:
//!     - Calls to such functions/methods call the original function/method for the original type,
//!       which means that preconditions are not taken into consideration. Use the [`forward`
//!       attribute][forward impl] to check the preconditions on these calls.
//!     - Because of the way they are implemented, it's currently possible for the name of these
//!       functions to clash with names in their surrounding module. This is unlikely to occur in
//!       regular usage, but possible. If you encounter such a case, please open an issue
//!       describing the problem.
//!     - Currently all type information for the `impl` block is discarded. This means that
//!       multiple non-overlapping (in the type system sense) `impl` blocks can overlap in an
//!       `extern_crate` annotated module.
//!
//!       ```rust,compile_fail
//!       mod a {
//!           pub(crate) struct X<T>(T);
//!
//!           impl X<bool> {
//!               pub(crate) fn foo() {}
//!           }
//!
//!           impl X<()> {
//!               pub(crate) fn foo() {}
//!           }
//!       }
//!
//!       #[pre::extern_crate(crate::a)]
//!       mod b {
//!           impl X<bool> {
//!               fn foo();
//!           }
//!
//!           impl X<()> {
//!               fn foo();
//!           }
//!       }
//!       #
//!       # fn main() {}
//!       ```
//!
//! # Understanding the error messages
//!
//! pre tries to be as helpful as possible in the error messages it gives. Unfortunately in some
//! cases pre does not have enough information to generate an error by itself, but has to rely on
//! rustc to do so later in the compilation. pre has very limited control over what these
//! messages look like.
//!
//! If you have trouble understanding these error messages, here is a little
//! overview what they look like and what they mean:
//!
//! ---
//!
//! ```text
//! error[E0061]: this function takes 2 arguments but 1 argument was supplied
//!  --> src/main.rs:7:5
//!   |
//! 3 |   #[pre(x > 41.9)]
//!   |  _______-
//! 4 | | fn foo(x: f32) {}
//!   | |_________________- defined here
//! ...
//! 7 |       foo(42.0);
//!   |       ^^^ ---- supplied 1 argument
//!   |       |
//!   |       expected 2 arguments
//! ```
//!
//! This error means that the function has preconditions, but they are not [`assure`d].
//!
//! To fix this error, find out what preconditions for the function are and whether they hold.
//! Once you're convinced that they hold, you can `assure` that to pre with an [`assure`
//! attribute] and explain in the `reason`, why you're sure that they hold. You should be able
//! to find the function preconditions in the documentation for the function.
//!
//! ---
//!
//! **nightly compiler error**
//! ```text
//! error[E0308]: mismatched types
//!   --> src/main.rs:8:5
//!    |
//! 8  | /     #[assure(
//! 9  | |         x > 41.0,
//! 10 | |         reason = "42.0 > 41.0"
//! 11 | |     )]
//!    | |______^ expected `"x > 41.9"`, found `"x > 41.0"`
//!    |
//!    = note: expected struct `std::marker::PhantomData<(pre::BooleanCondition<"x > 41.9">,)>`
//!               found struct `std::marker::PhantomData<(pre::BooleanCondition<"x > 41.0">,)>`
//! ```
//!
//! **stable compiler error**
//! ```text
//! error[E0560]: struct `foo` has no field named `_boolean_x_20_3e_2041_2e0`
//!  --> src/main.rs:9:9
//!   |
//! 9 |         x > 41.0,
//!   |         ^ help: a field with a similar name exists: `_boolean_x_20_3e_2041_2e9`
//! ```
//!
//! This error means that the preconditions that were [`assure`d] at the call site were different
//! from the preconditions at the function definition.
//!
//! Unfortunately the stable compiler error is not very readable for symbol heavy preconditions.
//! If have trouble reading these error messages, it is recommended to use the nightly compiler to
//! fix these errors. Once they are fixed, you can continue using the stable compiler.
//!
//! To fix this error, make sure that all `assure`d preconditions match the preconditions on the
//! function exactly.
//! Also when making changes to the `assure`d preconditions, make sure that they still hold.
//! You should be able to find the function preconditions in the documentation for the function.
//!
//! ---
//!
//! **nightly compiler error**
//! ```text
//! error[E0308]: mismatched types
//!   --> src/main.rs:9:5
//!    |
//! 9  | /     #[assure(
//! 10 | |         x > 41.9,
//! 11 | |         reason = "42.0 > 41.9"
//! 12 | |     )]
//!    | |______^ expected a tuple with 2 elements, found one with 1 element
//!    |
//!    = note: expected struct `std::marker::PhantomData<(pre::BooleanCondition<"x < 42.1">, pre::BooleanCondition<"x > 41.9">)>`
//!               found struct `std::marker::PhantomData<(pre::BooleanCondition<"x > 41.9">,)>`
//! ```
//!
//! **stable compiler error**
//! ```text
//! error[E0063]: missing field `_boolean_x_20_3c_2042_2e1` in initializer of `foo`
//!   --> src/main.rs:9:6
//!    |
//! 9  |       #[assure(
//!    |  ______^
//! 10 | |         x > 41.9,
//! 11 | |         reason = "42.0 > 41.9"
//! 12 | |     )]
//!    | |______^ missing `_boolean_x_20_3c_2042_2e1`
//! ```
//!
//! This error means that some, but not all, preconditions were [`assure`d] for a call.
//!
//! To fix this error, find out what preconditions you didn't consider yet and check whether they
//! hold. Once you're convinced that they hold, you can `assure` that to pre with an [`assure`
//! attribute] and explain in the `reason`, why you're sure that they hold. You should be able to
//! find the function preconditions in the documentation for the function.
//!
//! ---
//!
//! **nightly compiler error**
//! ```text
//! error[E0061]: this function takes 1 argument but 2 arguments were supplied
//!   --> src/main.rs:11:5
//!    |
//! 3  |   fn foo(x: f32) {}
//!    |   -------------- defined here
//! ...
//! 7  | /     #[assure(
//! 8  | |         x > 41.9,
//! 9  | |         reason = "42.0 > 41.9"
//! 10 | |     )]
//!    | |______- supplied 2 arguments
//! 11 |       foo(42.0);
//!    |       ^^^ ----
//!    |       |
//!    |       expected 1 argument
//! ```
//!
//! **stable compiler error**
//! ```text
//! error[E0574]: expected struct, variant or union type, found function `foo`
//!   --> src/main.rs:7:6
//!    |
//! 7  |       #[assure(
//!    |  ______^
//! 8  | |         x > 41.9,
//! 9  | |         reason = "42.0 > 41.9"
//! 10 | |     )]
//!    | |______^ not a struct, variant or union type
//!
//! error[E0061]: this function takes 1 argument but 2 arguments were supplied
//!   --> src/main.rs:11:5
//!    |
//! 3  |   fn foo(x: f32) {}
//!    |   -------------- defined here
//! ...
//! 7  |       #[assure(
//!    |  ______-
//! 8  | |         x > 41.9,
//! 9  | |         reason = "42.0 > 41.9"
//! 10 | |     )]
//!    | |______- supplied 2 arguments
//! 11 |       foo(42.0);
//!    |       ^^^ ----
//!    |       |
//!    |       expected 1 argument
//! ```
//!
//! This error means that one or more preconditions were [`assure`d] for a function that does
//! not have any preconditions.
//!
//! To fix this error, either [add the `assure`d preconditions as preconditions to the
//! function][`pre` attribute] or remove the `assure` attribute, if you added it in error.
//!
//! # Wording of preconditions
//!
//! While you can write any text you like in a [custom precondition][precondition syntax], it is
//! recommended to word them in a way that makes sense at both the definition and the call site.
//!
//! Therefore it is recommended not to write how things *should be*, but rather how they *are* when
//! everything is going well.
//!
//! > "the elements at `old_len..new_len` must be initialized"
//!
//! should instead be written as
//!
//! > "the elements at `old_len..new_len` **are** initialized"
//!
//! # Feature flags
//!
//! If you're planning on using pre in a library, you should consider how the increased
//! compile time might affect your users. If you're planning on making the preconditions part of
//! your public API there is not really a way around that.
//!
//! If you're only using the preconditions internally to check the correctness of your
//! implementation however, you can add pre as a `dev-dependency` and perform all those checks
//! behind a `cfg_attr`. This would not affect users of your library at all.
//!
//! Here is an example of that:
//!
//! ```toml
//! # Cargo.toml
//!
//! [dev-dependencies]
//! pre = "<latest version>"
//! ```
//!
//! ```rust
//! /* src/lib.rs */
//!
//! #[cfg(test)]
//! use pre::pre;
//!
//! #[cfg_attr(
//!     test,
//!     pre("your first condition"),
//!     pre("your second condition")
//! )]
//! fn some_non_pub_fn() {}
//!
//! #[cfg_attr(
//!     test,
//!     pre
//! )]
//! pub fn some_pub_fn() {
//!     #[cfg_attr(
//!         test,
//!         assure(
//!             "your first condition",
//!             reason = "your reason"
//!         ),
//!         assure(
//!             "your second condition",
//!             reason = "your reason"
//!         )
//!     )]
//!     some_non_pub_fn();
//! }
//! ```
//!
//! # Changing an existing code base to use pre
//!
//! One problem when changing a code base to use pre is that once a function has preconditions,
//! it needs them `assure`d everywhere.
//! For functions that are used a lot, it can be a big task to check and `assure` all call sites at
//! once.
//!
//! There are two ways to work around that, though one is currently only supported on the nightly
//! compiler.
//!
//! ## `extern_crate` for local items
//!
//! Suppose you have a function `some_module::some_fn` with a lot of uses that you want to change
//! to use pre without changing all call sites at once.
//!
//! ```rust
//! mod some_module {
//!     pub(crate) unsafe fn some_fn() {
//!         /* ... */
//!     }
//! }
//!
//! fn main() {
//!     use some_module::some_fn;
//!
//!     // Lots of uses
//!     unsafe {
//!         some_fn();
//!         some_fn();
//!     }
//! }
//! ```
//!
//! You can use the [`extern_crate` attribute] to create a version of `some_fn` with preconditions.
//!
//! ```rust
//! use pre::pre;
//!
//! mod some_module {
//!     pub(crate) unsafe fn some_fn() {
//!         /* ... */
//!     }
//! }
//!
//! #[pre::extern_crate(crate::some_module)]
//! mod pre_some_module {
//!     #[pre("some condition")]
//!     unsafe fn some_fn();
//! }
//!
//! #[pre]
//! fn main() {
//!     use some_module::some_fn;
//!
//!     unsafe {
//!         // Checks the preconditions of the function.
//!         #[forward(pre_some_module)]
//!         #[assure(
//!             "some condition",
//!             reason = "the reason you know the condition is true"
//!         )]
//!         some_fn();
//!
//!         // Does not check any preconditions as before the modifications.
//!         some_fn();
//!     }
//! }
//! ```
//!
//! When you've converted all call sites and you're ready to fully convert the function, you can
//! simply add the preconditions to the original function and remove the [`forward`
//! attributes][`forward` attribute].
//!
//! ```rust
//! use pre::pre;
//!
//! mod some_module {
//!     use pre::pre;
//!
//!     #[pre("some condition")]
//!     pub(crate) unsafe fn some_fn() {
//!         /* ... */
//!     }
//! }
//!
//! #[pre]
//! fn main() {
//!     use some_module::some_fn;
//!
//!     unsafe {
//!         #[assure(
//!             "some condition",
//!             reason = "the reason you know the condition is true"
//!         )]
//!         some_fn();
//!
//!         #[assure(
//!             "some condition",
//!             reason = "the (possibly other) reason you know the condition is true"
//!         )]
//!         some_fn();
//!     }
//! }
//! ```
//!
//! ## `"TODO"` as a reason
//!
//! **This paragraph only applies if you use the nightly compiler**, because it depends on the
//! [`proc_macro_diagnostic` feature].
//!
//! Using `"TODO"` as a reason in an `assure` attribute will issue a warning, to remind you of
//! checking why you believe the precondition holds.
//!
//! When changing a function to use pre, you can simply `assure` all its preconditions at all
//! call sites with `"TODO"` as the reason.
//! **Of course this does not make the use of the function any safer by itself.**
//! However it allows everything to compile again and have the compiler warnings remind you what
//! you need to `assure` still.
//!
//! Because the warnings only work on the nightly compiler, **usage of `"TODO"` as a reason is
//! discouraged when using the stable compiler**.
//!
//! [`pre` attribute]: attr.pre.html
//! [checking functionality]: attr.pre.html#checking-functionality
//! [precondition syntax]: attr.pre.html#precondition-syntax
//! [`assure` attribute]: attr.assure.html
//! [`assure`d]: attr.assure.html
//! [`extern_crate` attribute]: attr.extern_crate.html
//! [`forward` attribute]: attr.forward.html
//! [forward impl]: attr.forward.html#impl-call
//! [`cfg_attr` attribute]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
//! [`proc_macro_diagnostic` feature]: https://github.com/rust-lang/rust/issues/54140

#![allow(clippy::needless_doctest_main)]
#![cfg_attr(nightly, feature(adt_const_params))]
#![cfg_attr(nightly, allow(incomplete_features))]
#![cfg_attr(not(feature = "std"), no_std)]

/// Specify preconditions on functions and check that they are assured correctly for calls.
///
/// Its counterpart is the [`assure` attribute](attr.assure.html).
///
/// # Basic usage example
///
/// Suppose you have a precondition stating that a function `use_foo` must only be called after
/// some other initialization function `init_foo` was called. You could add that precondition to
/// the `use_foo` function as follows:
///
/// ```rust
/// use pre::pre;
///
/// #[pre("is only called after `init_foo` was called")]
/// fn use_foo(/* ... */) {
///     /* ... */
/// }
/// ```
///
/// To call the function `use_foo` now, you have to
/// [`assure`](attr.assure.html#basic-usage-example) that the precondition holds.
///
/// Note that the precondition states [*how things are if everything goes
/// well*](index.html#wording-of-preconditions) and not how they *should be*.
/// This makes it easier to read code calling the function.
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
///    - A proper alignment of the pointer.
///    - A valid initialized value for the pointee.
///
///    Also there are no guarantees about the size of the allocated object.
///    If there are no other preconditions about the size of the allocated object, usually the size
///    of a single value can be assumed.
/// 3. Proper alignment preconditions:
///
///    This precondition requires that a raw pointer has a proper alignment for its type.
///    More concretely for a `*const T` and `*mut T`, this means that the pointer must have an
///    alignment of `mem::align_of::<T>()`.
///
///    The syntax is `#[pre(proper_align(<ptr_name>))]`.
///
///    - `<ptr_name>`: The identifier of the pointer argument that must have a proper alignment.
///
///    ### Example
///
///    ```rust
///    # use pre::pre;
///    #
///    #[pre(proper_align(ptr_name))]
///    fn foo(ptr_name: *mut i32) {}
///    ```
/// 4. Boolean preconditions:
///
///    This precondition is a boolean expression that should evaluate to  `true` for the
///    precondition to hold.
///    By default a `debug_assert` statement is added to the function for such a precondition.
///    This can be disabled by a `#[pre(no_debug_assert)]` attribute.
///
///    The syntax is `#[pre(<expr>)]`.
///
///    - `<expr>`: A boolean expression that should evaluate to `true`.
///
///    ### Example
///
///    ```rust
///    # use pre::pre;
///    #
///    #[pre(a < b || b > 17)]
///    fn foo(a: i32, b: i32) {}
///    ```
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
///    fn foo() {} // foo will not have any documentation generated by pre.
///    ```
/// 4. Disable debug assertions for boolean preconditions.
///    ```rust
///    # use pre::pre;
///    #
///    #[pre(no_debug_assert)]
///    #[pre(old_val < new_val)]
///    fn foo() {} // foo will not have any `debug_assert`s generated by pre.
///    ```
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
/// the attributes of the item to prevent its generation.
pub use pre_proc_macro::pre;

/// Assure that a precondition holds.
///
/// This is the counterpart of the [`pre` attribute](attr.pre.html).
///
/// Currently this attribute does not work by itself.
/// It needs to be used inside of a context that is annotated by a `pre` attribute.
///
/// # Basic usage example
///
/// Suppose you want to call the function `use_foo` from the [example in the `pre` attribute
/// documentation](attr.pre.html).
///
/// First you need to find out what its preconditions are. You can do that by looking at its
/// documentation. For this example we can also look at the source code for the function and see
/// the precondition there.
///
/// It has one precondition:
///
/// - is only called after `init_foo` was called
///
/// Armed with that knowledge, we can write the code to call the function.
///
/// ```rust
/// use pre::pre;
/// #
/// # fn init_foo() {}
/// # #[pre("is only called after `init_foo` was called")]
/// # fn use_foo() {}
///
/// #[pre] // This is required, so `assure` works
/// fn main() {
///     // This call allows us to safely assure the precondition later.
///     init_foo(/* ... */);
///
///     #[assure(
///         "is only called after `init_foo` was called",
///         reason = "we just called `init_foo`"
///     )]
///     use_foo(/* ... */);
/// }
/// ```
///
/// # Terminology
///
/// The term `assure` was chosen because it most accurately describes the function of the
/// attribute.
///
/// There are no guarantees that a precondition holds, other than the fact that the programmer
/// promises that it does. The burden is still on the programmer in this case. pre only makes
/// sure that the programmer cannot forget that a precondition exists and that no precondition is
/// changed without the programmer noticing.
///
/// # Syntax
///
/// The basic syntax of the `assure` attribute is:
///
/// ```rust,ignore
/// #[assure(
///     <first precondition>,
///     reason = "<the reason why the first precondition can be assured>"
/// )]
/// #[assure(
///     <second precondition>,
///     reason = "<the reason why the second precondition can be assured>"
/// )]
/// #[assure(
///     <third precondition>,
///     reason = "<the reason why the third precondition can be assured>"
/// )]
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
///     #[assure(
///         valid_ptr(dst, w),
///         reason = "`dst` is created from a reference"
///     )]
///     unsafe { write_unaligned(&mut x, 2) };
/// #   assert_eq!(x, 2);
///
///     // Here the `std` segment of the path is replaced with `new_std`, so again
///     // `new_std::ptr::write_unaligned` is called.
///     // The same effect could be achieved without a `forward` attribute, by replacing the `std`
///     // in the path of the call with `new_std`.
///     #[forward(std -> new_std)]
///     #[assure(
///         valid_ptr(dst, w),
///         reason = "`dst` is created from a reference"
///     )]
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
///             #[pre(!ptr.is_null())]
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
///     #[assure(
///         !ptr.is_null(),
///         reason = "a reference is never null"
///     )]
///     let non_null = unsafe { new_std::ptr::NonNull::new_unchecked(&mut val) };
///
///     // The same thing also works when using the `NonNull` through the `std::ptr` path.
///     #[forward(impl new_std::ptr::NonNull)]
///     #[assure(
///         !ptr.is_null(),
///         reason = "a reference is never null"
///     )]
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
/// #[assure(
///     <some_condition>,
///     reason = <some_reason>
/// )]
/// v.some_method(some_arg);
/// ```
///
/// works similar to
///
/// ```rust,ignore
/// let v: SomeType<bool> = SomeType::new();
///
/// #[assure(
///     <some_condition>,
///     reason = <some_reason>
/// )]
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
/// This attribute can be used when a library has documented preconditions without using pre and
/// you want those preconditions to be checked by pre.
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
///         #[pre("an all-zero byte-pattern is valid for `T`")]
///         unsafe fn zeroed<T>() -> T;
///
///         impl<T> MaybeUninit<T> {
///             #[pre("the contained value is an initialized, valid value of `T`")]
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
///         "an all-zero byte-pattern is valid for `T`",
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
///         "the contained value is an initialized, valid value of `T`",
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

// Doctests don't work with this extern_crate, because there is a collision between it and `use
// pre::pre;`. Ideally this should use `cfg(doctest)`, but that currently doesn't work
// (https://github.com/rust-lang/rust/issues/67295). So instead testing for this crate is done
// without the default features and this `extern crate` is only present when either of the library
// features which need it are present.
// The disadvantage of this is that any tests that are feature gated on either of these features
// are not executed. This should not be a big issue however, since these features merely introduce
// declarations, which aren't testable in the first place.
//
// It is needed in the first place, because `pre::extern_crate` generates code that contains `use
// pre::pre;`.
#[cfg(any(feature = "core", feature = "std"))]
extern crate self as pre;

#[doc(hidden)]
#[cfg(any(feature = "core", feature = "std"))]
pub mod libs;

#[doc(inline)]
#[cfg(feature = "core")]
pub use libs::core;

#[cfg(feature = "alloc")]
extern crate alloc as alloc_lib;
#[doc(inline)]
#[cfg(feature = "alloc")]
pub use libs::alloc;

#[doc(inline)]
#[cfg(feature = "std")]
pub use libs::std;

cfg_if::cfg_if! {
    if #[cfg(nightly)] {
        // *WARNING* These types are not considered to be part of the public API and may change at
        // any time without notice.

        /// A condition that the pointer of name `PTR` is valid for `ACCESS_TYPE` accesses.
        #[doc(hidden)]
        pub struct ValidPtrCondition<const PTR: &'static str, const ACCESS_TYPE: &'static str>;

        /// A condition that the pointer of name `PTR` has a proper alignment for its type.
        #[doc(hidden)]
        pub struct ProperAlignCondition<const PTR: &'static str>;

        /// A boolean condition.
        #[doc(hidden)]
        pub struct BooleanCondition<const CONDITION: &'static str>;

        /// A custom condition.
        #[doc(hidden)]
        pub struct CustomCondition<const CONDITION: &'static str>;

    }
}
