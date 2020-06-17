use pre::{check_pre, pre};

#[pre("must be bar")]
#[pre("must also be baz")]
fn foo() {}

#[check_pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    #[assert_pre("must also be baz", reason = "is also baz")]
    foo()
}
