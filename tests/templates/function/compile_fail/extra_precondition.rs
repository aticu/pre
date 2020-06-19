use pre::pre;

#[pre("must be bar")]
fn foo() {}

#[pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    #[assert_pre("must be baz", reason = "is baz")]
    foo()
}
