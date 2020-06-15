# pre

`pre` is a work in progress [Rust](https://www.rust-lang.org/) crate that allows annotating functions and their call sites with preconditions and makes sure they match.
It is mostly intended for use with `unsafe` functions, as they have preconditions that cannot be checked at compile-time.
The main feature of `pre` is that probably incorrect code will not compile.

```rust
use pre::{pre, check_pre};

#[pre(condition(valid_ptr(ptr, r)))]
unsafe fn read_twice<T: Copy>(ptr: *const T) -> (T, T) {
    (std::ptr::read(ptr), std::ptr::read(ptr))
}

#[check_pre]
fn main() {
    let ptr: *const i32 = &42;

    let (a, b) = unsafe {
        #[assert_pre(condition(valid_ptr(ptr, r), reason = "the pointer is created from a reference"))]
        read_twice(ptr)
    };

    println!("First: {}", a);
    println!("Second: {}", b);
}
```

## Project Goals

These are to goals for the project in order of importance.
Not necessarily all of them will be possible at the same time.

- The library should produce errors at function call sites in the following cases:
  - At least one precondition was specified at the function definition, but none was specified at the call site.
  - A precondition was specified at the function definition, but that precondition was not specified at the call site.
  - A precondition was specified at the function call site, but that precondition was not specified at its definition.
  - A precondition was specified at the function call site, but no preconditions were specified at its definition.
- There should be no runtime overhead in the release version of the compiled binary.
- The order of the preconditions should not matter.
- The error messages should be easy to understand and inform the programmer how to fix the problem.
- There should be an option to insert the preconditions for the standard library functions without modifying the source, as if they were specified in the standard library. This would serve as a basis for doing code reviews using `unsafe` code.
- The library should ideally work on the stable Rust compiler, though this requirement likely contradicts the "nice error messages" requirement.
- The library should not have a large impact on compile times.
