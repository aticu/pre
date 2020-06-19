use pre::pre;

#[pre("must be bar")]
#[pre("must also be baz")]
fn foo() {}

#[pre]
fn main() {
    #[assert_pre("must be bar", reason = "is bar")]
    #[assert_pre("must also be baz", reason = "is also baz")]
    foo()
}
