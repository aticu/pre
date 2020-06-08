use pre::{check_pre, pre};

#[pre(condition("must be bar"), condition("must be baz"))]
fn foo() {}

#[check_pre]
fn main() {
    #[assert_pre(condition("must be bar", reason = "is bar"))]
    foo()
}
