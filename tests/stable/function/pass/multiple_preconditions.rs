use pre::{check_pre, pre};

#[pre(condition("must be bar"))]
#[pre(condition("must also be baz"))]
fn foo() {}

#[check_pre]
fn main() {
    #[assert_pre(condition("must be bar", reason = "is bar"))]
    #[assert_pre(condition("must also be baz", reason = "is also baz"))]
    foo()
}
