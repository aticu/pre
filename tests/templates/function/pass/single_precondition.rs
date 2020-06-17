use pre::{check_pre, pre};

#[pre("must be bar")]
fn foo() {}

#[check_pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    foo()
}
