#![no_std]

use pre::pre;

#[pre("foo")]
fn foo() {}

#[pre]
fn main() {
    #[assure("foo", reason = "is foo")]
    foo();
}
