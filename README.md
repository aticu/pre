# pre

`pre` is a [Rust](https://www.rust-lang.org/) library to help programmers correctly uphold preconditions for function calls.
It is mostly intended for use with `unsafe` functions, as they have preconditions that cannot be checked at compile-time.

## Motivation

Sometimes functions or methods have preconditions that cannot be ensured in the type system and
cannot be guarded against at runtime.
The most prominent example of functions like that are `unsafe` functions.
When used correctly, `unsafe` functions are used to ["declare the existence of contracts the
compiler can't check"](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html).
These contracts are the preconditions for the function call.
Failing to uphold them usually results in a violation of memory safety and undefined behavior.

Currently the most used scheme for dealing with these preconditions on `unsafe` functions is to
mention them in the `Safety` section of the function's documentation. Programmers using the
function then have to check what they have to ensure to call the function correctly. The
programmer that uses the function may then leave a comment next to the function, describing why
the call is safe (why the preconditions hold).

This approach is even advertised by the compiler (as of 1.44.1) when using an `unsafe` function
outside of an `unsafe` block:

```text
note: consult the function's documentation for information on how to avoid undefined behavior
```

There are multiple points of failure with this approach. Among others these are:

1. The original function could not document what the preconditions are.
2. The programmer using the function could forget to look at the documented preconditions.
3. The programmer using the function could overlook one or more of the preconditions. This is
   not unlikely, if multiple preconditions are documented in a single long paragraph.
4. The programmer could make a mistake when checking whether the preconditions hold.
5. An update could change the preconditions that the function requires and users of the
   function could easily miss that fact.
6. The programmer could forget (or choose not to) document why the preconditions hold when
   calling the function, making it easier for others to make a mistake when later changing the
   call or code around it.

This library cannot guard against all of these problems, especially not against 1 and 4.
It however attempts to help against 2, 3, 5 and 6.

## The approach

This library works by allowing programmers to specify preconditions on functions they write in
a unified format.
Those preconditions are then transformed into an additional function argument.
Callers of the function then specify the same preconditions at the call site, along with a
reason why they believe the precondition is upheld.
If the preconditions don't match or are not specified, the function will have invalid arguments
and the code will not compile. This should protect against problems 2, 3 and 5 from above.
Because a reason is required at the function call site, problem 6 is at least partially guarded
against, though programmers could still choose to not put too much effort into the reason.

The signature of the functions is changed by adding a single *zero-sized* parameter.
**This means that when compiled using the release mode, there is no run-time cost for these
checks. They are a zero-cost abstraction.**

## Usage

The basic usage for the `pre` crate looks like this:

```rust
use pre::pre;

#[pre("`arg` is a meaningful value")]
fn foo(arg: i32) {
    assert_eq!(arg, 42);
}

#[pre] // Enables `assure`ing preconditions inside the function
fn main() {
    #[assure("`arg` is a meaningful value", reason = "42 is very meaningful")]
    foo(42);
}
```

The [`pre` attribute](https://docs.rs/pre/0.1.0/pre/attr.pre.html) serves to specify
preconditions (on `foo`) and to enable usage of the `assure` attribute (on `main`).
To learn why the second usage is necessary, read the [paragraph about the checking
functionality](https://docs.rs/pre/0.1.0/pre/attr.pre.html#checking-functionality) on the
documentation of the `pre` attribute.

With the [`assure` attribute](https://docs.rs/pre/0.1.0/pre/attr.assure.html) the
programmer assures that the precondition was checked by them and is upheld.
Without the `assure` attribute, the code would fail to compile.

```rust,compile_fail
use pre::pre;

#[pre("`arg` must have a meaningful value")]
fn foo(arg: i32) {
    assert_eq!(arg, 42);
}

fn main() {
    foo(42);
//  ^^^^^^^-- this errors
}
```

**The precondition inside the `assure` attribute must be exactly equal to the precondition
inside the `pre` attribute at the function definition for the code to compile.**
The order of the preconditions, if there are multiple, does not matter however.

## Known Limitations

Unfortunately while `pre` tries to be as helpful as possible, there are some situations in
which it is quite limited in what it can do:

- There is more than one form of `unsafe` code. `pre` currently exclusively focuses on `unsafe`
  functions.
- While `pre` does work on the stable compiler, there are quite a few things that only work
  when using the nightly compiler.

  These are the main differences between the nightly version and the stable version (there are
  other minor ones):
    - **Preconditions on functions in `impl` blocks only work on nightly.**

      This does not apply to `impl` blocks inside of an `extern_crate` annotated module. These
      have their own limitations though (see below).
    - Warnings from `pre` are only possible on nightly.
    - Errors can reference multiple locations providing better suggestions and messages on
      nightly.
- Since `pre` works by adding an additional argument to a function, it changes the function
  signature. That won't make a difference in many cases, but if you use function pointers or
  pass a function as an argument, it will have a different type from what it appears to be.
- Because attribute macros are not supported for expressions and statements on the current
  stable compiler, functions that contain an `assure` attribute must have at least one `pre`
  attribute, though it could be empty:
  [`#[pre]`](https://docs.rs/pre/0.1.0/pre/attr.pre.html#checking-functionality).
- There are multiple limitations for functions and methods defined in a module which is
  annotated with the [`extern_crate`
  attribute](https://docs.rs/pre/0.1.0/pre/attr.extern_crate.html) or has a parent that is:
    - Calls to such functions/methods call the original function/method for the original type,
      which means that preconditions are not taken into consideration. Use the [`forward`
      attribute](https://docs.rs/pre/0.1.0/pre/attr.forward.html#impl-call) to check the
      preconditions on these calls.
    - Because of the way they are implemented, it's currently possible for the name of these
      functions to clash with names in their surrounding module.  This is unlikely to occur in
      regular usage, but possible. If you encounter such a case, please open an issue
      describing the problem.
    - Currently all type information for the `impl` block is discarded. This means that
      multiple non-overlapping (in the type system sense) `impl` blocks can overlap in an
      `extern_crate` annotated module.

## Understanding the error messages

`pre` tries to be as helpful as possible in the error messages it gives. Unfortunately in some
cases `pre` does not have enough information to generate an error by itself, but has to rely on
rustc to do so later in the compilation. `pre` has very limited control over what these
messages look like.

If you have trouble understanding these error messages, here is a little
overview what they look like and what they mean:

---

```text
error[E0061]: this function takes 2 arguments but 1 argument was supplied
 --> src/main.rs:7:5
  |
3 |   #[pre(x > 41.9)]
  |  _______-
4 | | fn foo(x: f32) {}
  | |_________________- defined here
...
7 |       foo(42.0);
  |       ^^^ ---- supplied 1 argument
  |       |
  |       expected 2 arguments
```

This error means that the function has preconditions, but they are not
[`assure`d](https://docs.rs/pre/0.1.0/pre/attr.assure.html).

To fix this error, find out what preconditions for the function are and whether they hold.
Once you're convinced that they hold, you can `assure` that to `pre` with an [`assure`
attribute](https://docs.rs/pre/0.1.0/pre/attr.assure.html) and explain in the `reason`,
why you're sure that they hold. You should be able to find the function preconditions in the
documentation for the function.

---

**nightly compiler error**
```text
error[E0308]: mismatched types
  --> src/main.rs:8:5
   |
8  | /     #[assure(
9  | |         x > 41.0,
10 | |         reason = "42.0 > 41.0"
11 | |     )]
   | |______^ expected `"x > 41.9"`, found `"x > 41.0"`
   |
   = note: expected struct `std::marker::PhantomData<(pre::BooleanCondition<"x > 41.9">,)>`
              found struct `std::marker::PhantomData<(pre::BooleanCondition<"x > 41.0">,)>`
```

**stable compiler error**
```text
error[E0560]: struct `foo` has no field named `_boolean_x_20_3e_2041_2e0`
 --> src/main.rs:9:9
  |
9 |         x > 41.0,
  |         ^ help: a field with a similar name exists: `_boolean_x_20_3e_2041_2e9`
```

This error means that the preconditions that were
[`assure`d](https://docs.rs/pre/0.1.0/pre/attr.assure.html) at the call site were different
from the preconditions at the function definition.

Unfortunately the stable compiler error is not very readable for symbol heavy preconditions.
If have trouble reading these error messages, it is recommended to use the nightly compiler to
fix these errors. Once they are fixed, you can continue using the stable compiler.

To fix this error, make sure that all `assure`d preconditions match the preconditions on the
function exactly.
Also when making changes to the `assure`d preconditions, make sure that they still hold.
You should be able to find the function preconditions in the documentation for the function.

---

**nightly compiler error**
```text
error[E0308]: mismatched types
  --> src/main.rs:9:5
   |
9  | /     #[assure(
10 | |         x > 41.9,
11 | |         reason = "42.0 > 41.9"
12 | |     )]
   | |______^ expected a tuple with 2 elements, found one with 1 element
   |
   = note: expected struct `std::marker::PhantomData<(pre::BooleanCondition<"x < 42.1">, pre::BooleanCondition<"x > 41.9">)>`
              found struct `std::marker::PhantomData<(pre::BooleanCondition<"x > 41.9">,)>`
```

**stable compiler error**
```text
error[E0063]: missing field `_boolean_x_20_3c_2042_2e1` in initializer of `foo`
  --> src/main.rs:13:5
   |
13 |     foo(42.0);
   |     ^^^ missing `_boolean_x_20_3c_2042_2e1`
```

This error means that some, but not all, preconditions were
[`assure`d](https://docs.rs/pre/0.1.0/pre/attr.assure.html) for a call.

To fix this error, find out what preconditions you didn't consider yet and check whether they
hold. Once you're convinced that they hold, you can `assure` that to `pre` with an [`assure`
attribute](https://docs.rs/pre/0.1.0/pre/attr.assure.html) and explain in the `reason`, why
you're sure that they hold. You should be able to find the function preconditions in the
documentation for the function.

---

**nightly compiler error**
```text
error[E0061]: this function takes 1 argument but 2 arguments were supplied
  --> src/main.rs:11:5
   |
3  |   fn foo(x: f32) {}
   |   -------------- defined here
...
7  | /     #[assure(
8  | |         x > 41.9,
9  | |         reason = "42.0 > 41.9"
10 | |     )]
   | |______- supplied 2 arguments
11 |       foo(42.0);
   |       ^^^ ----
   |       |
   |       expected 1 argument
```

**stable compiler error**
```text
error[E0574]: expected struct, variant or union type, found function `foo`
  --> src/main.rs:11:5
   |
11 |     foo(42.0);
   |     ^^^ not a struct, variant or union type

error[E0061]: this function takes 1 argument but 2 arguments were supplied
  --> src/main.rs:11:5
   |
3  |   fn foo(x: f32) {}
   |   -------------- defined here
...
7  | /     #[assure(
8  | |         x > 41.9,
9  | |         reason = "42.0 > 41.9"
10 | |     )]
11 | |     foo(42.0);
   | |     ^^- ----
   | |_____|_|
   |       | supplied 2 arguments
   |       expected 1 argument
```

This error means that one or more preconditions were
[`assure`d](https://docs.rs/pre/0.1.0/pre/attr.assure.html) for a function that does not
have any preconditions.

To fix this error, either [add the `assure`d preconditions as preconditions to the
function](https://docs.rs/pre/0.1.0/pre/attr.pre.html) or remove the `assure` attribute,
if you added it in error.

## Background

This library is developed for my bachelor's thesis titled "Implementierung und Evaluation von Vorbedinungskommunikation für unsafe Code in Rust" (in German).
The second part of the thesis focuses on evaluating whether such a library is useful and whether the benefits are worth the additional effort.

I'd be very grateful if you open an issue with any feedback that you have on this library, as that helps my evaluation efforts.
